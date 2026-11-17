import { DEFAULT_NETWORK_CONFIG, NetworkConfig } from './network'

/** Sparse network-level defaults shared with the backend patch model. */
export interface NetworkConfigPatch {
  values: NetworkConfig
  set_fields: string[]
  clear_fields: string[]
}

/**
 * Field paths the network settings may default. Mirrors the backend
 * `NetworkConfigField::NETWORK_DEFAULTS` list; member-only fields (VPN
 * portal clients and keys) and central-owned fields stay out.
 */
export const NETWORK_DEFAULT_FIELD_PATHS: ReadonlyArray<readonly string[]> = [
  ['listener_urls'],
  ['latency_first'],
  ['dev_name'],
  ['use_smoltcp'],
  ['disable_ipv6'],
  ['enable_kcp_proxy'],
  ['disable_kcp_input'],
  ['disable_p2p'],
  ['bind_device'],
  ['no_tun'],
  ['enable_exit_node'],
  ['relay_all_peer_rpc'],
  ['multi_thread'],
  ['enable_relay_network_whitelist'],
  ['relay_network_whitelist'],
  ['enable_manual_routes'],
  ['routes'],
  ['exit_nodes'],
  ['proxy_forward_by_system'],
  ['disable_encryption'],
  ['enable_socks5'],
  ['socks5_port'],
  ['disable_udp_hole_punching'],
  ['mtu'],
  ['mapped_listeners'],
  ['enable_magic_dns'],
  ['enable_private_mode'],
  ['enable_quic_proxy'],
  ['disable_quic_input'],
  ['port_forwards'],
  ['disable_sym_hole_punching'],
  ['p2p_only'],
  ['disable_tcp_hole_punching'],
  ['lazy_p2p'],
  ['need_p2p'],
  ['instance_recv_bps_limit'],
  ['disable_upnp'],
  ['ipv6_public_addr_auto'],
  ['disable_relay_data'],
  ['enable_udp_broadcast_relay'],
  ['prefer_peer_relay'],
  ['enable_udp_fec'],
  ['enable_udp_fec_kcp'],
  ['enable_udp_fec_quic'],
  ['proxy_cidrs'],
  ['vpn_portal_config', 'enabled'],
  ['vpn_portal_config', 'wireguard_listen'],
]

function readPath(source: any, path: readonly string[]): any {
  let current = source
  for (const part of path) {
    if (current == null || typeof current !== 'object') return undefined
    current = current[part]
  }
  return current
}

function writePath(target: any, path: readonly string[], value: any): void {
  let current = target
  for (const part of path.slice(0, -1)) {
    if (current[part] == null || typeof current[part] !== 'object') {
      current[part] = {}
    }
    current = current[part]
  }
  current[path[path.length - 1]!] = value
}

function isEmptyListValue(value: any): boolean {
  return Array.isArray(value) && value.length === 0
}

/** A draft the editor starts from: explicit values only where set. */
export function draftFromPatch(patch: NetworkConfigPatch): NetworkConfig {
  const draft: any = DEFAULT_NETWORK_CONFIG()
  for (const field of patch.set_fields) {
    const path = NETWORK_DEFAULT_FIELD_PATHS.find(
      (candidate) => candidate.join('.') === field,
    )
    if (!path) continue
    writePath(draft, path, readPath(patch.values as any, path))
  }
  return draft as NetworkConfig
}

/**
 * Diff an edited draft against its baseline into a sparse patch. Fields the
 * editor did not touch stay unset, so unset is never confused with an
 * explicit false or an explicit empty list.
 */
export function patchFromDraft(
  baseline: NetworkConfig,
  draft: NetworkConfig,
): NetworkConfigPatch {
  const setFields: string[] = []
  const baselineJson = baseline as any
  const draftJson = draft as any
  const values: any = DEFAULT_NETWORK_CONFIG()
  for (const path of NETWORK_DEFAULT_FIELD_PATHS) {
    const key = path.join('.')
    const before = readPath(baselineJson, path)
    const after = readPath(draftJson, path)
    const changed =
      JSON.stringify(before ?? null) !== JSON.stringify(after ?? null)
    if (!changed) continue
    if (after === undefined || after === null || isEmptyListValue(after)) {
      if (before !== undefined && before !== null && !isEmptyListValue(before)) {
        // Cleared back to empty: keep it as an explicit empty value only for
        // list fields, otherwise drop the override entirely.
        if (Array.isArray(after) || Array.isArray(before)) {
          writePath(values, path, [])
          setFields.push(key)
        }
        continue
      }
      continue
    }
    writePath(values, path, after)
    setFields.push(key)
  }
  return { values, set_fields: setFields, clear_fields: [] }
}
