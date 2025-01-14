//! Database operations related to character data
//!
//! Methods in this module should remain private to the persistence module -
//! database updates and loading are communicated via requests to the
//! [`CharacterLoader`] and [`CharacterUpdater`] while results/responses are
//! polled and handled each server tick.
extern crate diesel;

use super::{error::Error, models::*, schema, VelorenTransaction};
use crate::{
    comp,
    comp::{item::MaterialStatManifest, Inventory},
    persistence::{
        character::conversions::{
            convert_body_from_database, convert_body_to_database_json,
            convert_character_from_database, convert_inventory_from_database_items,
            convert_items_to_database_items, convert_loadout_from_database_items,
            convert_skill_groups_to_database, convert_skills_to_database,
            convert_stats_from_database, convert_waypoint_from_database_json,
            convert_waypoint_to_database_json,
        },
        character_loader::{CharacterCreationResult, CharacterDataResult, CharacterListResult},
        error::Error::DatabaseError,
        PersistedComponents,
    },
};
use common::character::{CharacterId, CharacterItem, MAX_CHARACTERS_PER_PLAYER};
use core::ops::Range;
use diesel::{prelude::*, sql_query, sql_types::BigInt};
use std::{collections::VecDeque, sync::Arc};
use tracing::{error, trace, warn};

/// Private module for very tightly coupled database conversion methods.  In
/// general, these have many invariants that need to be maintained when they're
/// called--do not assume it's safe to make these public!
mod conversions;

pub(crate) type EntityId = i64;

const CHARACTER_PSEUDO_CONTAINER_DEF_ID: &str = "veloren.core.pseudo_containers.character";
const INVENTORY_PSEUDO_CONTAINER_DEF_ID: &str = "veloren.core.pseudo_containers.inventory";
const LOADOUT_PSEUDO_CONTAINER_DEF_ID: &str = "veloren.core.pseudo_containers.loadout";
const INVENTORY_PSEUDO_CONTAINER_POSITION: &str = "inventory";
const LOADOUT_PSEUDO_CONTAINER_POSITION: &str = "loadout";
const WORLD_PSEUDO_CONTAINER_ID: EntityId = 1;

#[derive(Clone, Copy)]
struct CharacterContainers {
    inventory_container_id: EntityId,
    loadout_container_id: EntityId,
}

/// BFS the inventory/loadout to ensure that each is topologically sorted in the
/// sense required by convert_inventory_from_database_items to support recursive
/// items
pub fn load_items_bfs(connection: VelorenTransaction, root: i64) -> Result<Vec<Item>, Error> {
    use schema::item::dsl::*;
    let mut items = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_front(root);
    while let Some(id) = queue.pop_front() {
        let frontier = item
            .filter(parent_container_item_id.eq(id))
            .load::<Item>(&*connection)?;
        for i in frontier.iter() {
            queue.push_back(i.item_id);
        }
        items.extend(frontier);
    }
    Ok(items)
}

/// Load stored data for a character.
///
/// After first logging in, and after a character is selected, we fetch this
/// data for the purpose of inserting their persisted data for the entity.
pub fn load_character_data(
    requesting_player_uuid: String,
    char_id: CharacterId,
    connection: VelorenTransaction,
    msm: &MaterialStatManifest,
) -> CharacterDataResult {
    use schema::{body::dsl::*, character::dsl::*, skill_group::dsl::*};

    let character_containers = get_pseudo_containers(connection, char_id)?;

    let inventory_items = load_items_bfs(connection, character_containers.inventory_container_id)?;
    let loadout_items = load_items_bfs(connection, character_containers.loadout_container_id)?;

    let character_data = character
        .filter(
            schema::character::dsl::character_id
                .eq(char_id)
                .and(player_uuid.eq(requesting_player_uuid)),
        )
        .first::<Character>(&*connection)?;

    let char_body = body
        .filter(schema::body::dsl::body_id.eq(char_id))
        .first::<Body>(&*connection)?;

    let char_waypoint = character_data.waypoint.as_ref().and_then(|x| {
        match convert_waypoint_from_database_json(&x) {
            Ok(w) => Some(w),
            Err(e) => {
                warn!(
                    "Error reading waypoint from database for character ID {}, error: {}",
                    char_id, e
                );
                None
            },
        }
    });

    let skill_data = schema::skill::dsl::skill
        .filter(schema::skill::dsl::entity_id.eq(char_id))
        .load::<Skill>(&*connection)?;

    let skill_group_data = skill_group
        .filter(schema::skill_group::dsl::entity_id.eq(char_id))
        .load::<SkillGroup>(&*connection)?;

    Ok((
        convert_body_from_database(&char_body)?,
        convert_stats_from_database(character_data.alias, &skill_data, &skill_group_data),
        convert_inventory_from_database_items(
            character_containers.inventory_container_id,
            &inventory_items,
            character_containers.loadout_container_id,
            &loadout_items,
            msm,
        )?,
        char_waypoint,
    ))
}

/// Loads a list of characters belonging to the player. This data is a small
/// subset of the character's data, and is used to render the character and
/// their level in the character list.
///
/// In the event that a join fails, for a character (i.e. they lack an entry for
/// stats, body, etc...) the character is skipped, and no entry will be
/// returned.
pub fn load_character_list(
    player_uuid_: &str,
    connection: VelorenTransaction,
    msm: &MaterialStatManifest,
) -> CharacterListResult {
    use schema::{body::dsl::*, character::dsl::*};

    let result = character
        .filter(player_uuid.eq(player_uuid_))
        .order(schema::character::dsl::character_id.desc())
        .load::<Character>(&*connection)?;

    result
        .iter()
        .map(|character_data| {
            let char = convert_character_from_database(character_data);

            let db_body = body
                .filter(schema::body::dsl::body_id.eq(character_data.character_id))
                .first::<Body>(&*connection)?;

            let char_body = convert_body_from_database(&db_body)?;

            let loadout_container_id = get_pseudo_container_id(
                connection,
                character_data.character_id,
                LOADOUT_PSEUDO_CONTAINER_POSITION,
            )?;

            let loadout_items = load_items_bfs(connection, loadout_container_id)?;

            let loadout =
                convert_loadout_from_database_items(loadout_container_id, &loadout_items, msm)?;

            Ok(CharacterItem {
                character: char,
                body: char_body,
                inventory: Inventory::new_with_loadout(loadout),
            })
        })
        .collect()
}

pub fn create_character(
    uuid: &str,
    character_alias: &str,
    persisted_components: PersistedComponents,
    connection: VelorenTransaction,
    msm: &MaterialStatManifest,
) -> CharacterCreationResult {
    use schema::item::dsl::*;

    check_character_limit(uuid, connection)?;

    use schema::{body, character, skill_group};

    let (body, stats, inventory, waypoint) = persisted_components;

    // Fetch new entity IDs for character, inventory and loadout
    let mut new_entity_ids = get_new_entity_ids(connection, |next_id| next_id + 3)?;

    // Create pseudo-container items for character
    let character_id = new_entity_ids.next().unwrap();
    let inventory_container_id = new_entity_ids.next().unwrap();
    let loadout_container_id = new_entity_ids.next().unwrap();

    let pseudo_containers = vec![
        Item {
            stack_size: 1,
            item_id: character_id,
            parent_container_item_id: WORLD_PSEUDO_CONTAINER_ID,
            item_definition_id: CHARACTER_PSEUDO_CONTAINER_DEF_ID.to_owned(),
            position: character_id.to_string(),
        },
        Item {
            stack_size: 1,
            item_id: inventory_container_id,
            parent_container_item_id: character_id,
            item_definition_id: INVENTORY_PSEUDO_CONTAINER_DEF_ID.to_owned(),
            position: INVENTORY_PSEUDO_CONTAINER_POSITION.to_owned(),
        },
        Item {
            stack_size: 1,
            item_id: loadout_container_id,
            parent_container_item_id: character_id,
            item_definition_id: LOADOUT_PSEUDO_CONTAINER_DEF_ID.to_owned(),
            position: LOADOUT_PSEUDO_CONTAINER_POSITION.to_owned(),
        },
    ];
    let pseudo_container_count = diesel::insert_into(item)
        .values(pseudo_containers)
        .execute(&*connection)?;

    if pseudo_container_count != 3 {
        return Err(Error::OtherError(format!(
            "Error inserting initial pseudo containers for character id {} (expected 3, actual {})",
            character_id, pseudo_container_count
        )));
    }

    let skill_set = stats.skill_set;

    // Insert body record
    let new_body = Body {
        body_id: character_id,
        body_data: convert_body_to_database_json(&body)?,
        variant: "humanoid".to_string(),
    };

    let body_count = diesel::insert_into(body::table)
        .values(&new_body)
        .execute(&*connection)?;

    if body_count != 1 {
        return Err(Error::OtherError(format!(
            "Error inserting into body table for char_id {}",
            character_id
        )));
    }

    // Insert character record
    let new_character = NewCharacter {
        character_id,
        player_uuid: uuid,
        alias: &character_alias,
        waypoint: convert_waypoint_to_database_json(waypoint),
    };
    let character_count = diesel::insert_into(character::table)
        .values(&new_character)
        .execute(&*connection)?;

    if character_count != 1 {
        return Err(Error::OtherError(format!(
            "Error inserting into character table for char_id {}",
            character_id
        )));
    }

    let db_skill_groups = convert_skill_groups_to_database(character_id, skill_set.skill_groups);
    let skill_groups_count = diesel::insert_into(skill_group::table)
        .values(&db_skill_groups)
        .execute(&*connection)?;

    if skill_groups_count != 1 {
        return Err(Error::OtherError(format!(
            "Error inserting into skill_group table for char_id {}",
            character_id
        )));
    }

    // Insert default inventory and loadout item records
    let mut inserts = Vec::new();

    get_new_entity_ids(connection, |mut next_id| {
        let inserts_ = convert_items_to_database_items(
            loadout_container_id,
            &inventory,
            inventory_container_id,
            &mut next_id,
        );
        inserts = inserts_;
        next_id
    })?;

    let expected_inserted_count = inserts.len();
    let inserted_items = inserts
        .into_iter()
        .map(|item_pair| item_pair.model)
        .collect::<Vec<_>>();
    let inserted_count = diesel::insert_into(item)
        .values(&inserted_items)
        .execute(&*connection)?;

    if expected_inserted_count != inserted_count {
        return Err(Error::OtherError(format!(
            "Expected insertions={}, actual={}, for char_id {}--unsafe to continue transaction.",
            expected_inserted_count, inserted_count, character_id
        )));
    }

    load_character_list(uuid, connection, msm).map(|list| (character_id, list))
}

/// Delete a character. Returns the updated character list.
pub fn delete_character(
    requesting_player_uuid: &str,
    char_id: CharacterId,
    connection: VelorenTransaction,
    msm: &MaterialStatManifest,
) -> CharacterListResult {
    use schema::{body::dsl::*, character::dsl::*, skill::dsl::*, skill_group::dsl::*};

    // Load the character to delete - ensures that the requesting player
    // owns the character
    let _character_data = character
        .filter(
            schema::character::dsl::character_id
                .eq(char_id)
                .and(player_uuid.eq(requesting_player_uuid)),
        )
        .first::<Character>(&*connection)?;

    // Delete skills
    diesel::delete(skill_group.filter(schema::skill_group::dsl::entity_id.eq(char_id)))
        .execute(&*connection)?;

    diesel::delete(skill.filter(schema::skill::dsl::entity_id.eq(char_id)))
        .execute(&*connection)?;

    // Delete character
    let character_count = diesel::delete(
        character
            .filter(schema::character::dsl::character_id.eq(char_id))
            .filter(player_uuid.eq(requesting_player_uuid)),
    )
    .execute(&*connection)?;

    if character_count != 1 {
        return Err(Error::OtherError(format!(
            "Error deleting from character table for char_id {}",
            char_id
        )));
    }

    // Delete body
    let body_count = diesel::delete(body.filter(schema::body::dsl::body_id.eq(char_id)))
        .execute(&*connection)?;

    if body_count != 1 {
        return Err(Error::OtherError(format!(
            "Error deleting from body table for char_id {}",
            char_id
        )));
    }

    // Delete all items, recursively walking all containers starting from the
    // "character" pseudo-container that is the root for all items owned by
    // a character.
    let item_count = diesel::sql_query(format!(
        "
    WITH RECURSIVE
    parents AS (
        SELECT  item_id
        FROM    item
        WHERE   item.item_id = {} -- Item with character id is the character pseudo-container
        UNION ALL
        SELECT  item.item_id
        FROM    item,
                parents
        WHERE   item.parent_container_item_id = parents.item_id
    )
    DELETE
    FROM    item
    WHERE EXISTS (SELECT 1 FROM parents WHERE parents.item_id = item.item_id)",
        char_id
    ))
    .execute(&*connection)?;

    if item_count < 3 {
        return Err(Error::OtherError(format!(
            "Error deleting from item table for char_id {} (expected at least 3 deletions, found \
             {})",
            char_id, item_count
        )));
    }

    load_character_list(requesting_player_uuid, connection, msm)
}

/// Before creating a character, we ensure that the limit on the number of
/// characters has not been exceeded
pub fn check_character_limit(uuid: &str, connection: VelorenTransaction) -> Result<(), Error> {
    use diesel::dsl::count_star;
    use schema::character::dsl::*;

    let character_count = character
        .select(count_star())
        .filter(player_uuid.eq(uuid))
        .load::<i64>(&*connection)?;

    match character_count.first() {
        Some(count) => {
            if count < &(MAX_CHARACTERS_PER_PLAYER as i64) {
                Ok(())
            } else {
                Err(Error::CharacterLimitReached)
            }
        },
        _ => Ok(()),
    }
}

/// NOTE: This relies heavily on serializability to work correctly.
///
/// The count function takes the starting entity id, and returns the desired
/// count of new entity IDs.
///
/// These are then inserted into the entities table.
fn get_new_entity_ids(
    conn: VelorenTransaction,
    mut max: impl FnMut(i64) -> i64,
) -> Result<Range<EntityId>, Error> {
    use super::schema::entity::dsl::*;

    #[derive(QueryableByName)]
    struct NextEntityId {
        #[sql_type = "BigInt"]
        entity_id: i64,
    }

    // The sqlite_sequence table is used here to avoid reusing entity IDs for
    // deleted entities. This table always contains the highest used ID for each
    // AUTOINCREMENT column in a SQLite database.
    let next_entity_id = sql_query(
        "
        SELECT  seq + 1 AS entity_id
        FROM    sqlite_sequence
        WHERE name = 'entity'",
    )
    .load::<NextEntityId>(&*conn)?
    .pop()
    .ok_or_else(|| Error::OtherError("No rows returned for sqlite_sequence query ".to_string()))?
    .entity_id;

    let max_entity_id = max(next_entity_id);

    // Create a new range of IDs and insert them into the entity table
    let new_ids: Range<EntityId> = next_entity_id..max_entity_id;

    let new_entities: Vec<Entity> = new_ids.clone().map(|x| Entity { entity_id: x }).collect();

    let actual_count = diesel::insert_into(entity)
        .values(&new_entities)
        .execute(&*conn)?;

    if actual_count != new_entities.len() {
        return Err(Error::OtherError(format!(
            "Error updating entity table: expected to add the range {:?}) to entities, but actual \
             insertions={}",
            new_ids, actual_count
        )));
    }

    trace!(
        "Created {} new persistence entity_ids: {}",
        new_ids.end - new_ids.start,
        new_ids
            .clone()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );
    Ok(new_ids)
}

/// Fetches the pseudo_container IDs for a character
fn get_pseudo_containers(
    connection: VelorenTransaction,
    character_id: CharacterId,
) -> Result<CharacterContainers, Error> {
    let character_containers = CharacterContainers {
        loadout_container_id: get_pseudo_container_id(
            connection,
            character_id,
            LOADOUT_PSEUDO_CONTAINER_POSITION,
        )?,
        inventory_container_id: get_pseudo_container_id(
            connection,
            character_id,
            INVENTORY_PSEUDO_CONTAINER_POSITION,
        )?,
    };

    Ok(character_containers)
}

fn get_pseudo_container_id(
    connection: VelorenTransaction,
    character_id: CharacterId,
    pseudo_container_position: &str,
) -> Result<EntityId, Error> {
    use super::schema::item::dsl::*;
    match item
        .select(item_id)
        .filter(
            parent_container_item_id
                .eq(character_id)
                .and(position.eq(pseudo_container_position)),
        )
        .first::<EntityId>(&*connection)
    {
        Ok(id) => Ok(id),
        Err(e) => {
            error!(
                ?e,
                ?character_id,
                ?pseudo_container_position,
                "Failed to retrieve pseudo container ID"
            );
            Err(DatabaseError(e))
        },
    }
}

pub fn update(
    char_id: CharacterId,
    char_stats: comp::Stats,
    inventory: comp::Inventory,
    char_waypoint: Option<comp::Waypoint>,
    connection: VelorenTransaction,
) -> Result<Vec<Arc<common::comp::item::ItemId>>, Error> {
    use super::schema::{character::dsl::*, item::dsl::*, skill_group::dsl::*};

    let pseudo_containers = get_pseudo_containers(connection, char_id)?;

    let mut upserts = Vec::new();

    // First, get all the entity IDs for any new items, and identify which slots to
    // upsert and which ones to delete.
    get_new_entity_ids(connection, |mut next_id| {
        let upserts_ = convert_items_to_database_items(
            pseudo_containers.loadout_container_id,
            &inventory,
            pseudo_containers.inventory_container_id,
            &mut next_id,
        );
        upserts = upserts_;
        next_id
    })?;

    // Next, delete any slots we aren't upserting.
    trace!("Deleting items for character_id {}", char_id);
    let mut existing_item_ids: Vec<i64> = vec![
        pseudo_containers.inventory_container_id,
        pseudo_containers.loadout_container_id,
    ];
    for it in load_items_bfs(connection, pseudo_containers.inventory_container_id)? {
        existing_item_ids.push(it.item_id);
    }
    for it in load_items_bfs(connection, pseudo_containers.loadout_container_id)? {
        existing_item_ids.push(it.item_id);
    }
    let existing_items = parent_container_item_id.eq_any(existing_item_ids);
    let non_upserted_items = item_id.ne_all(
        upserts
            .iter()
            .map(|item_pair| item_pair.model.item_id)
            .collect::<Vec<_>>(),
    );

    let delete_count = diesel::delete(item.filter(existing_items.and(non_upserted_items)))
        .execute(&*connection)?;
    trace!("Deleted {} items", delete_count);

    // Upsert items
    let expected_upsert_count = upserts.len();
    let mut upserted_comps = Vec::new();
    if expected_upsert_count > 0 {
        let (upserted_items, upserted_comps_): (Vec<_>, Vec<_>) = upserts
            .into_iter()
            .map(|model_pair| {
                debug_assert_eq!(
                    model_pair.model.item_id,
                    model_pair.comp.load().unwrap().get() as i64
                );
                (model_pair.model, model_pair.comp)
            })
            .unzip();
        upserted_comps = upserted_comps_;
        trace!(
            "Upserting items {:?} for character_id {}",
            upserted_items,
            char_id
        );

        // When moving inventory items around, foreign key constraints on
        // `parent_container_item_id` can be temporarily violated by one upsert, but
        // restored by another upsert. Deferred constraints allow SQLite to check this
        // when committing the transaction. The `defer_foreign_keys` pragma treats the
        // foreign key constraints as deferred for the next transaction (it turns itself
        // off at the commit boundary). https://sqlite.org/foreignkeys.html#fk_deferred
        connection.execute("PRAGMA defer_foreign_keys = ON;")?;
        let upsert_count = diesel::replace_into(item)
            .values(&upserted_items)
            .execute(&*connection)?;
        trace!("upsert_count: {}", upsert_count);
        if upsert_count != expected_upsert_count {
            return Err(Error::OtherError(format!(
                "Expected upsertions={}, actual={}, for char_id {}--unsafe to continue \
                 transaction.",
                expected_upsert_count, upsert_count, char_id
            )));
        }
    }

    let char_skill_set = char_stats.skill_set;

    let db_skill_groups = convert_skill_groups_to_database(char_id, char_skill_set.skill_groups);

    diesel::replace_into(skill_group)
        .values(&db_skill_groups)
        .execute(&*connection)?;

    let db_skills = convert_skills_to_database(char_id, char_skill_set.skills);

    let delete_count = diesel::delete(
        schema::skill::dsl::skill.filter(
            schema::skill::dsl::entity_id.eq(char_id).and(
                schema::skill::dsl::skill_type.ne_all(
                    db_skills
                        .iter()
                        .map(|x| x.skill_type.clone())
                        .collect::<Vec<_>>(),
                ),
            ),
        ),
    )
    .execute(&*connection)?;
    trace!("Deleted {} skills", delete_count);

    diesel::replace_into(schema::skill::dsl::skill)
        .values(&db_skills)
        .execute(&*connection)?;

    let db_waypoint = convert_waypoint_to_database_json(char_waypoint);
    let waypoint_count =
        diesel::update(character.filter(schema::character::dsl::character_id.eq(char_id)))
            .set(waypoint.eq(db_waypoint))
            .execute(&*connection)?;

    if waypoint_count != 1 {
        return Err(Error::OtherError(format!(
            "Error updating character table for char_id {}",
            char_id
        )));
    }

    Ok(upserted_comps)
}
