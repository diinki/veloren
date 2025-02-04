use crate::{client::Client, metrics::NetworkRequestMetrics, presence::Presence};
use common::{
    comp::Pos,
    event::{EventBus, ServerEvent},
    terrain::{TerrainChunkSize, TerrainGrid},
    vol::RectVolSize,
};
use common_ecs::{Job, Origin, Phase, System};
use common_net::msg::{ClientGeneral, ServerGeneral};
use specs::{Entities, Join, Read, ReadExpect, ReadStorage};
use tracing::{debug, trace};

/// This system will handle new messages from clients
#[derive(Default)]
pub struct Sys;
impl<'a> System<'a> for Sys {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        Read<'a, EventBus<ServerEvent>>,
        ReadExpect<'a, TerrainGrid>,
        ReadExpect<'a, NetworkRequestMetrics>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Presence>,
        ReadStorage<'a, Client>,
    );

    const NAME: &'static str = "msg::terrain";
    const ORIGIN: Origin = Origin::Server;
    const PHASE: Phase = Phase::Create;

    fn run(
        _job: &mut Job<Self>,
        (
            entities,
            server_event_bus,
            terrain,
            network_metrics,
            positions,
            presences,
            clients,
        ): Self::SystemData,
    ) {
        let mut server_emitter = server_event_bus.emitter();

        for (entity, client, maybe_presence) in (&entities, &clients, (&presences).maybe()).join() {
            let _ = super::try_recv_all(client, 5, |client, msg| {
                let presence = match maybe_presence {
                    Some(g) => g,
                    None => {
                        debug!(?entity, "client is not in_game, ignoring msg");
                        trace!(?msg, "ignored msg content");
                        if matches!(msg, ClientGeneral::TerrainChunkRequest { .. }) {
                            network_metrics.chunks_request_dropped.inc();
                        }
                        return Ok(());
                    },
                };
                match msg {
                    ClientGeneral::TerrainChunkRequest { key } => {
                        let in_vd = if let Some(pos) = positions.get(entity) {
                            pos.0.xy().map(|e| e as f64).distance_squared(
                                key.map(|e| e as f64 + 0.5)
                                    * TerrainChunkSize::RECT_SIZE.map(|e| e as f64),
                            ) < ((presence.view_distance as f64 - 1.0 + 2.5 * 2.0_f64.sqrt())
                                * TerrainChunkSize::RECT_SIZE.x as f64)
                                .powi(2)
                        } else {
                            true
                        };
                        if in_vd {
                            match terrain.get_key(key) {
                                Some(chunk) => {
                                    network_metrics.chunks_served_from_memory.inc();
                                    client.send(ServerGeneral::TerrainChunkUpdate {
                                        key,
                                        chunk: Ok(Box::new(chunk.clone())),
                                    })?
                                },
                                None => {
                                    network_metrics.chunks_generation_triggered.inc();
                                    server_emitter.emit(ServerEvent::ChunkRequest(entity, key))
                                },
                            }
                        } else {
                            network_metrics.chunks_request_dropped.inc();
                        }
                    },
                    _ => tracing::error!("not a client_terrain msg"),
                }
                Ok(())
            });
        }
    }
}
