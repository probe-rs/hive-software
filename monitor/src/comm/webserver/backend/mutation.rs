//! The graphql mutation
use std::sync::Arc;

use anyhow::anyhow;
use async_graphql::{Context, Object, Result as GraphQlResult};
use comm_types::{
    hardware::{ProbeInfo, ProbeState, TargetState},
    ipc::{HiveProbeData, HiveTargetData},
};
use probe_rs::Probe;

use crate::database::{keys, CborDb, HiveDb};

use super::model::{AssignProbeResponse, AssignTargetResponse, FlatProbeState, State};

pub(in crate::comm::webserver) struct BackendMutation;

#[Object]
impl BackendMutation {
    /// Assigns a target to a given position. This does only update the data in the DB. To apply the changes into the runtime use the update mutation to reinitialize the testrack
    async fn assign_target<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(validator(maximum = 7))] tss_pos: usize,
        #[graphql(validator(maximum = 3))] target_pos: usize,
        target_name: String,
    ) -> GraphQlResult<AssignTargetResponse> {
        let db = ctx.data::<Arc<HiveDb>>().unwrap();

        let mut assigned = db
            .config_tree
            .c_get::<HiveTargetData>(keys::config::ASSIGNED_TARGETS)
            .unwrap()
            .expect("DB not initialized");

        if assigned[tss_pos].is_none() {
            return Err(anyhow!("Cannot assign target to tss without daughterboard"))?;
        }

        let target_state: TargetState = target_name.clone().into();

        assigned[tss_pos].as_mut().unwrap()[target_pos] = target_state;

        db.config_tree
            .c_insert(keys::config::ASSIGNED_TARGETS, &assigned)
            .unwrap();

        Ok(AssignTargetResponse {
            tss_pos: tss_pos as u8,
            target_pos: target_pos as u8,
            target_name,
        })
    }

    async fn assign_probe<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(validator(maximum = 3))] probe_pos: usize,
        probe_state: FlatProbeState,
    ) -> GraphQlResult<AssignProbeResponse> {
        let db = ctx.data::<Arc<HiveDb>>().unwrap();

        let mut assigned = db
            .config_tree
            .c_get::<HiveProbeData>(keys::config::ASSIGNED_PROBES)
            .unwrap()
            .expect("DB not initialized");

        if probe_state.state == State::Known {
            if assigned.iter().any(|assigned_probe| {
                if let ProbeState::Known(assigned_probe_info) = assigned_probe {
                    return assigned_probe_info.identifier
                        == probe_state.data.as_ref().unwrap().identifier
                        && assigned_probe_info.serial_number
                            == probe_state.data.as_ref().unwrap().serial_number;
                }
                false
            }) {
                return Err(anyhow!(
                    "Cannot reassign a probe which is already assigned to a testchannel"
                ))?;
            }
        }

        if probe_state.state == State::Known {
            let probes = Probe::list_all();

            let mut probe_found = false;

            for probe in probes.into_iter() {
                if probe.identifier == probe_state.data.as_ref().unwrap().identifier {
                    probe_found = true;
                    assigned[probe_pos] = ProbeState::Known(ProbeInfo {
                        identifier: probe.identifier,
                        vendor_id: probe.vendor_id,
                        product_id: probe.product_id,
                        serial_number: probe.serial_number,
                        hid_interface: probe.hid_interface,
                    });
                }
            }

            if !probe_found {
                return Err(anyhow!("Could not detect the provided probe"))?;
            }
        } else {
            assigned[probe_pos] = match probe_state.state {
                State::Known => unreachable!(),
                State::Unknown => ProbeState::Unknown,
                State::NotConnected => ProbeState::NotConnected,
            };
        }

        db.config_tree
            .c_insert(keys::config::ASSIGNED_PROBES, &assigned)
            .unwrap();

        Ok(AssignProbeResponse {
            probe_pos: probe_pos as u8,
            data: probe_state,
        })
    }
}
