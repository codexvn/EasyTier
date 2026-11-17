use std::collections::BTreeSet;

use easytier::common::config::{NetworkConfig, NetworkConfigExt, TomlConfigLoader};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkConfigField {
    ListenerUrls,
    LatencyFirst,
    DevName,
    UseSmoltcp,
    DisableIpv6,
    EnableKcpProxy,
    DisableKcpInput,
    DisableP2p,
    BindDevice,
    NoTun,
    EnableExitNode,
    RelayAllPeerRpc,
    MultiThread,
    EnableRelayNetworkWhitelist,
    RelayNetworkWhitelist,
    EnableManualRoutes,
    Routes,
    ExitNodes,
    ProxyForwardBySystem,
    DisableEncryption,
    EnableSocks5,
    Socks5Port,
    DisableUdpHolePunching,
    Mtu,
    MappedListeners,
    EnableMagicDns,
    EnablePrivateMode,
    EnableQuicProxy,
    DisableQuicInput,
    PortForwards,
    DisableSymHolePunching,
    P2pOnly,
    DisableTcpHolePunching,
    LazyP2p,
    NeedP2p,
    InstanceRecvBpsLimit,
    DisableUpnp,
    Ipv6PublicAddrAuto,
    DisableRelayData,
    EnableUdpBroadcastRelay,
    PreferPeerRelay,
    EnableUdpFec,
    EnableUdpFecKcp,
    EnableUdpFecQuic,
    ProxyCidrs,
    VpnPortalEnabled,
    VpnPortalWireguardListen,
    VpnPortalWireguardPrivateKey,
    VpnPortalClients,
}

impl NetworkConfigField {
    pub const ALL: &[Self] = &[
        Self::ListenerUrls,
        Self::LatencyFirst,
        Self::DevName,
        Self::UseSmoltcp,
        Self::DisableIpv6,
        Self::EnableKcpProxy,
        Self::DisableKcpInput,
        Self::DisableP2p,
        Self::BindDevice,
        Self::NoTun,
        Self::EnableExitNode,
        Self::RelayAllPeerRpc,
        Self::MultiThread,
        Self::EnableRelayNetworkWhitelist,
        Self::RelayNetworkWhitelist,
        Self::EnableManualRoutes,
        Self::Routes,
        Self::ExitNodes,
        Self::ProxyForwardBySystem,
        Self::DisableEncryption,
        Self::EnableSocks5,
        Self::Socks5Port,
        Self::DisableUdpHolePunching,
        Self::Mtu,
        Self::MappedListeners,
        Self::EnableMagicDns,
        Self::EnablePrivateMode,
        Self::EnableQuicProxy,
        Self::DisableQuicInput,
        Self::PortForwards,
        Self::DisableSymHolePunching,
        Self::P2pOnly,
        Self::DisableTcpHolePunching,
        Self::LazyP2p,
        Self::NeedP2p,
        Self::InstanceRecvBpsLimit,
        Self::DisableUpnp,
        Self::Ipv6PublicAddrAuto,
        Self::DisableRelayData,
        Self::EnableUdpBroadcastRelay,
        Self::PreferPeerRelay,
        Self::EnableUdpFec,
        Self::EnableUdpFecKcp,
        Self::EnableUdpFecQuic,
        Self::ProxyCidrs,
        Self::VpnPortalEnabled,
        Self::VpnPortalWireguardListen,
        Self::VpnPortalWireguardPrivateKey,
        Self::VpnPortalClients,
    ];

    pub const NETWORK_DEFAULTS: &[Self] = &[
        Self::ListenerUrls,
        Self::LatencyFirst,
        Self::DevName,
        Self::UseSmoltcp,
        Self::DisableIpv6,
        Self::EnableKcpProxy,
        Self::DisableKcpInput,
        Self::DisableP2p,
        Self::BindDevice,
        Self::NoTun,
        Self::EnableExitNode,
        Self::RelayAllPeerRpc,
        Self::MultiThread,
        Self::EnableRelayNetworkWhitelist,
        Self::RelayNetworkWhitelist,
        Self::EnableManualRoutes,
        Self::Routes,
        Self::ExitNodes,
        Self::ProxyForwardBySystem,
        Self::DisableEncryption,
        Self::EnableSocks5,
        Self::Socks5Port,
        Self::DisableUdpHolePunching,
        Self::Mtu,
        Self::MappedListeners,
        Self::EnableMagicDns,
        Self::EnablePrivateMode,
        Self::EnableQuicProxy,
        Self::DisableQuicInput,
        Self::PortForwards,
        Self::DisableSymHolePunching,
        Self::P2pOnly,
        Self::DisableTcpHolePunching,
        Self::LazyP2p,
        Self::NeedP2p,
        Self::InstanceRecvBpsLimit,
        Self::DisableUpnp,
        Self::Ipv6PublicAddrAuto,
        Self::DisableRelayData,
        Self::EnableUdpBroadcastRelay,
        Self::PreferPeerRelay,
        Self::EnableUdpFec,
        Self::EnableUdpFecKcp,
        Self::EnableUdpFecQuic,
        Self::ProxyCidrs,
        Self::VpnPortalEnabled,
        Self::VpnPortalWireguardListen,
    ];

    pub fn is_network_default(self) -> bool {
        Self::NETWORK_DEFAULTS.contains(&self)
    }

    /// Whether the field's leaf is a repeated list. An absent list key in a
    /// serialized configuration means an explicit empty list, not a missing
    /// value, so list fields never fail presence validation.
    fn is_list(self) -> bool {
        matches!(
            self,
            Self::ListenerUrls
                | Self::RelayNetworkWhitelist
                | Self::Routes
                | Self::ExitNodes
                | Self::MappedListeners
                | Self::PortForwards
                | Self::ProxyCidrs
                | Self::VpnPortalClients
        )
    }

    fn path(self) -> &'static [&'static str] {
        match self {
            Self::ListenerUrls => &["listener_urls"],
            Self::LatencyFirst => &["latency_first"],
            Self::DevName => &["dev_name"],
            Self::UseSmoltcp => &["use_smoltcp"],
            Self::DisableIpv6 => &["disable_ipv6"],
            Self::EnableKcpProxy => &["enable_kcp_proxy"],
            Self::DisableKcpInput => &["disable_kcp_input"],
            Self::DisableP2p => &["disable_p2p"],
            Self::BindDevice => &["bind_device"],
            Self::NoTun => &["no_tun"],
            Self::EnableExitNode => &["enable_exit_node"],
            Self::RelayAllPeerRpc => &["relay_all_peer_rpc"],
            Self::MultiThread => &["multi_thread"],
            Self::EnableRelayNetworkWhitelist => &["enable_relay_network_whitelist"],
            Self::RelayNetworkWhitelist => &["relay_network_whitelist"],
            Self::EnableManualRoutes => &["enable_manual_routes"],
            Self::Routes => &["routes"],
            Self::ExitNodes => &["exit_nodes"],
            Self::ProxyForwardBySystem => &["proxy_forward_by_system"],
            Self::DisableEncryption => &["disable_encryption"],
            Self::EnableSocks5 => &["enable_socks5"],
            Self::Socks5Port => &["socks5_port"],
            Self::DisableUdpHolePunching => &["disable_udp_hole_punching"],
            Self::Mtu => &["mtu"],
            Self::MappedListeners => &["mapped_listeners"],
            Self::EnableMagicDns => &["enable_magic_dns"],
            Self::EnablePrivateMode => &["enable_private_mode"],
            Self::EnableQuicProxy => &["enable_quic_proxy"],
            Self::DisableQuicInput => &["disable_quic_input"],
            Self::PortForwards => &["port_forwards"],
            Self::DisableSymHolePunching => &["disable_sym_hole_punching"],
            Self::P2pOnly => &["p2p_only"],
            Self::DisableTcpHolePunching => &["disable_tcp_hole_punching"],
            Self::LazyP2p => &["lazy_p2p"],
            Self::NeedP2p => &["need_p2p"],
            Self::InstanceRecvBpsLimit => &["instance_recv_bps_limit"],
            Self::DisableUpnp => &["disable_upnp"],
            Self::Ipv6PublicAddrAuto => &["ipv6_public_addr_auto"],
            Self::DisableRelayData => &["disable_relay_data"],
            Self::EnableUdpBroadcastRelay => &["enable_udp_broadcast_relay"],
            Self::PreferPeerRelay => &["prefer_peer_relay"],
            Self::EnableUdpFec => &["enable_udp_fec"],
            Self::EnableUdpFecKcp => &["enable_udp_fec_kcp"],
            Self::EnableUdpFecQuic => &["enable_udp_fec_quic"],
            Self::ProxyCidrs => &["proxy_cidrs"],
            Self::VpnPortalEnabled => &["vpn_portal_config", "enabled"],
            Self::VpnPortalWireguardListen => &["vpn_portal_config", "wireguard_listen"],
            Self::VpnPortalWireguardPrivateKey => &["vpn_portal_config", "wireguard_private_key"],
            Self::VpnPortalClients => &["vpn_portal_config", "clients"],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NetworkConfigPatch {
    #[serde(default)]
    pub values: NetworkConfig,
    #[serde(default)]
    pub set_fields: BTreeSet<NetworkConfigField>,
    #[serde(default)]
    pub clear_fields: BTreeSet<NetworkConfigField>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NetworkConfigPatchError {
    #[error("configuration field is both set and cleared: {0:?}")]
    ConflictingField(NetworkConfigField),
    #[error("configuration value is missing for field: {0:?}")]
    MissingValue(NetworkConfigField),
    #[error("member-only field cannot be set as a network default: {0:?}")]
    MemberOnlyField(NetworkConfigField),
    #[error("invalid sparse network configuration: {0}")]
    Invalid(String),
}

pub fn current_core_defaults() -> Result<NetworkConfig, NetworkConfigPatchError> {
    NetworkConfig::new_from_config(TomlConfigLoader::default())
        .map_err(|error| NetworkConfigPatchError::Invalid(format!("{error:#}")))
}

impl NetworkConfigPatch {
    pub fn validate(&self) -> Result<(), NetworkConfigPatchError> {
        if let Some(field) = self
            .set_fields
            .intersection(&self.clear_fields)
            .next()
            .copied()
        {
            return Err(NetworkConfigPatchError::ConflictingField(field));
        }
        let values = to_json(&self.values)?;
        for field in &self.set_fields {
            if !field.is_list() && field_value(&values, *field).is_none_or(|value| value.is_null())
            {
                return Err(NetworkConfigPatchError::MissingValue(*field));
            }
        }
        Ok(())
    }

    pub fn validate_network_defaults(&self) -> Result<(), NetworkConfigPatchError> {
        if let Some(field) = self
            .set_fields
            .iter()
            .chain(self.clear_fields.iter())
            .find(|field| !field.is_network_default())
        {
            return Err(NetworkConfigPatchError::MemberOnlyField(*field));
        }
        self.validate()
    }

    pub fn sanitize(&self) -> Result<Self, NetworkConfigPatchError> {
        self.validate()?;
        let source = to_json(&self.values)?;
        let mut values = to_json(&NetworkConfig::default())?;
        for field in &self.set_fields {
            let value = field_value(&source, *field)
                .filter(|value| !value.is_null())
                .or_else(|| field.is_list().then(|| Value::Array(Vec::new())))
                .ok_or(NetworkConfigPatchError::MissingValue(*field))?;
            set_field(&mut values, *field, value)?;
        }
        Ok(Self {
            values: from_json(values)?,
            set_fields: self.set_fields.clone(),
            clear_fields: self.clear_fields.clone(),
        })
    }

    pub fn from_legacy_snapshot(
        config: &NetworkConfig,
        inherit_fields: &BTreeSet<NetworkConfigField>,
    ) -> Result<Self, NetworkConfigPatchError> {
        let values = to_json(config)?;
        let set_fields = NetworkConfigField::ALL
            .iter()
            .copied()
            .filter(|field| {
                !inherit_fields.contains(field)
                    && field_value(&values, *field).is_some_and(|value| !value.is_null())
            })
            .collect();
        Self {
            values: config.clone(),
            set_fields,
            clear_fields: BTreeSet::new(),
        }
        .sanitize()
    }

    pub fn apply_to(
        &self,
        target: &mut NetworkConfig,
        core_defaults: &NetworkConfig,
    ) -> Result<(), NetworkConfigPatchError> {
        self.validate()?;
        let mut target_json = to_json(target)?;
        let values = to_json(&self.values)?;
        let defaults = to_json(core_defaults)?;
        for field in &self.clear_fields {
            if let Some(value) = field_value(&defaults, *field) {
                set_field(&mut target_json, *field, value)?;
            } else {
                remove_field(&mut target_json, *field)?;
            }
        }
        for field in &self.set_fields {
            let value = field_value(&values, *field)
                .filter(|value| !value.is_null())
                .or_else(|| field.is_list().then(|| Value::Array(Vec::new())))
                .ok_or(NetworkConfigPatchError::MissingValue(*field))?;
            set_field(&mut target_json, *field, value)?;
        }
        *target = from_json(target_json)?;
        Ok(())
    }

    pub fn reset_to_defaults(
        core_defaults: &NetworkConfig,
    ) -> Result<Self, NetworkConfigPatchError> {
        let defaults = to_json(core_defaults)?;
        let mut values = to_json(&NetworkConfig::default())?;
        let mut set_fields = BTreeSet::new();
        let mut clear_fields = BTreeSet::new();
        for field in NetworkConfigField::NETWORK_DEFAULTS {
            if let Some(value) = field_value(&defaults, *field).filter(|value| !value.is_null()) {
                set_field(&mut values, *field, value)?;
                set_fields.insert(*field);
            } else {
                clear_fields.insert(*field);
            }
        }
        Ok(Self {
            values: from_json(values)?,
            set_fields,
            clear_fields,
        })
    }

    pub fn matching_network_fields(
        config: &NetworkConfig,
        defaults: &NetworkConfig,
    ) -> Result<BTreeSet<NetworkConfigField>, NetworkConfigPatchError> {
        let config = to_json(config)?;
        let defaults = to_json(defaults)?;
        Ok(NetworkConfigField::NETWORK_DEFAULTS
            .iter()
            .copied()
            .filter(|field| field_value(&config, *field) == field_value(&defaults, *field))
            .collect())
    }

    pub fn remove_fields_equal_to(
        &mut self,
        effective_defaults: &NetworkConfig,
        core_defaults: &NetworkConfig,
    ) -> Result<(), NetworkConfigPatchError> {
        self.validate()?;
        let values = to_json(&self.values)?;
        let effective_defaults = to_json(effective_defaults)?;
        let core_defaults = to_json(core_defaults)?;
        self.set_fields.retain(|field| {
            !field.is_network_default()
                || field_value(&values, *field) != field_value(&effective_defaults, *field)
        });
        self.clear_fields.retain(|field| {
            !field.is_network_default()
                || field_value(&core_defaults, *field) != field_value(&effective_defaults, *field)
        });
        *self = self.sanitize()?;
        Ok(())
    }
}

fn to_json<T: Serialize>(value: &T) -> Result<Value, NetworkConfigPatchError> {
    serde_json::to_value(value).map_err(|error| NetworkConfigPatchError::Invalid(error.to_string()))
}

fn from_json<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, NetworkConfigPatchError> {
    serde_json::from_value(value)
        .map_err(|error| NetworkConfigPatchError::Invalid(error.to_string()))
}

fn field_value(value: &Value, field: NetworkConfigField) -> Option<Value> {
    let mut current = value;
    for part in field.path() {
        current = current.get(*part)?;
    }
    Some(current.clone())
}

fn set_field(
    root: &mut Value,
    field: NetworkConfigField,
    value: Value,
) -> Result<(), NetworkConfigPatchError> {
    let path = field.path();
    let mut current = root;
    for part in &path[..path.len() - 1] {
        if !current.get(*part).is_some_and(Value::is_object) {
            current[*part] = Value::Object(Map::new());
        }
        current = current
            .get_mut(*part)
            .ok_or_else(|| NetworkConfigPatchError::Invalid(format!("missing object {part}")))?;
    }
    current[path[path.len() - 1]] = value;
    Ok(())
}

fn remove_field(
    root: &mut Value,
    field: NetworkConfigField,
) -> Result<(), NetworkConfigPatchError> {
    let path = field.path();
    let mut current = root;
    for part in &path[..path.len() - 1] {
        let Some(next) = current.get_mut(*part) else {
            return Ok(());
        };
        current = next;
    }
    if let Some(object) = current.as_object_mut() {
        object.remove(path[path.len() - 1]);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use easytier::proto::api::manage::{VpnPortalClientConfig, VpnPortalConfig};

    #[test]
    fn sparse_patch_distinguishes_unset_false_and_empty_lists() {
        let patch = NetworkConfigPatch {
            values: NetworkConfig {
                enable_udp_fec: Some(false),
                proxy_cidrs: Vec::new(),
                ..Default::default()
            },
            set_fields: [
                NetworkConfigField::EnableUdpFec,
                NetworkConfigField::ProxyCidrs,
            ]
            .into_iter()
            .collect(),
            clear_fields: BTreeSet::new(),
        };
        let sanitized = patch.sanitize().unwrap();
        assert_eq!(sanitized.values.enable_udp_fec, Some(false));
        assert!(sanitized.values.proxy_cidrs.is_empty());
        assert!(
            sanitized
                .set_fields
                .contains(&NetworkConfigField::ProxyCidrs)
        );
        assert!(sanitized.clear_fields.is_empty());
    }

    #[test]
    fn network_defaults_do_not_replace_member_portal_clients_or_keys() {
        let patch = NetworkConfigPatch {
            values: NetworkConfig {
                vpn_portal_config: Some(VpnPortalConfig {
                    enabled: Some(true),
                    wireguard_listen: "0.0.0.0:11013".to_owned(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            set_fields: [
                NetworkConfigField::VpnPortalEnabled,
                NetworkConfigField::VpnPortalWireguardListen,
            ]
            .into_iter()
            .collect(),
            clear_fields: BTreeSet::new(),
        };
        let mut target = NetworkConfig {
            vpn_portal_config: Some(VpnPortalConfig {
                wireguard_private_key: Some("member-key".to_owned()),
                clients: vec![VpnPortalClientConfig {
                    name: "member-client".to_owned(),
                    virtual_ip: "10.0.0.3/24".to_owned(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        patch
            .apply_to(&mut target, &NetworkConfig::default())
            .unwrap();
        let portal = target.vpn_portal_config.unwrap();
        assert_eq!(portal.enabled, Some(true));
        assert_eq!(portal.wireguard_listen, "0.0.0.0:11013");
        assert_eq!(portal.wireguard_private_key.as_deref(), Some("member-key"));
        assert_eq!(portal.clients[0].name, "member-client");
    }

    #[test]
    fn network_defaults_reject_member_only_fields_and_unset_values() {
        let member_only = NetworkConfigPatch {
            values: NetworkConfig {
                vpn_portal_config: Some(VpnPortalConfig {
                    clients: vec![VpnPortalClientConfig::default()],
                    ..Default::default()
                }),
                ..Default::default()
            },
            set_fields: [NetworkConfigField::VpnPortalClients].into_iter().collect(),
            clear_fields: BTreeSet::new(),
        };
        assert!(matches!(
            member_only.validate_network_defaults(),
            Err(NetworkConfigPatchError::MemberOnlyField(
                NetworkConfigField::VpnPortalClients
            ))
        ));

        let missing_value = NetworkConfigPatch {
            values: NetworkConfig::default(),
            set_fields: [NetworkConfigField::EnableUdpFec].into_iter().collect(),
            clear_fields: BTreeSet::new(),
        };
        assert!(matches!(
            missing_value.validate(),
            Err(NetworkConfigPatchError::MissingValue(
                NetworkConfigField::EnableUdpFec
            ))
        ));
    }

    #[test]
    fn reset_to_defaults_covers_shared_fields_and_marks_missing_as_clear() {
        let core_defaults = current_core_defaults().unwrap();
        let patch = NetworkConfigPatch::reset_to_defaults(&core_defaults).unwrap();
        assert!(patch.set_fields.is_disjoint(&patch.clear_fields));
        assert_eq!(
            patch.set_fields.len() + patch.clear_fields.len(),
            NetworkConfigField::NETWORK_DEFAULTS.len()
        );
        let mut effective = core_defaults.clone();
        patch.apply_to(&mut effective, &core_defaults).unwrap();
        // Applying the reset patch to the defaults it came from is a no-op.
        let original = serde_json::to_value(&core_defaults).unwrap();
        let applied = serde_json::to_value(&effective).unwrap();
        for field in NetworkConfigField::NETWORK_DEFAULTS {
            assert_eq!(
                field_value(&original, *field),
                field_value(&applied, *field),
                "reset changed field {field:?}"
            );
        }
    }

    #[test]
    fn matching_network_fields_flags_equal_values_and_remove_drops_duplicates() {
        let core_defaults = current_core_defaults().unwrap();
        let defaults = NetworkConfigPatch::reset_to_defaults(&core_defaults).unwrap();
        let mut effective = core_defaults.clone();
        defaults.apply_to(&mut effective, &core_defaults).unwrap();
        // Core defaults carry no explicit values for these fields, so a reset
        // expresses them as clear markers; sample from that set.
        let sample_clear_field = *defaults.clear_fields.iter().next().unwrap();

        let mut duplicate = effective.clone();
        duplicate.proxy_cidrs = vec!["10.90.0.0/24".to_owned()];
        let matched = NetworkConfigPatch::matching_network_fields(&duplicate, &effective).unwrap();
        assert!(matched.contains(&sample_clear_field));
        assert!(!matched.contains(&NetworkConfigField::ProxyCidrs));

        let mut patch = NetworkConfigPatch {
            values: duplicate,
            set_fields: [
                NetworkConfigField::MappedListeners,
                NetworkConfigField::ProxyCidrs,
            ]
            .into_iter()
            .collect(),
            clear_fields: BTreeSet::new(),
        };
        patch
            .remove_fields_equal_to(&effective, &core_defaults)
            .unwrap();
        // The empty list equal to the reset default drops out; the
        // member-specific proxy CIDR list stays.
        assert!(
            !patch
                .set_fields
                .contains(&NetworkConfigField::MappedListeners)
        );
        assert!(patch.set_fields.contains(&NetworkConfigField::ProxyCidrs));
        assert_eq!(patch.values.proxy_cidrs, vec!["10.90.0.0/24".to_owned()]);
    }
}
