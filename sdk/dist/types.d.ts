/**
 * OP Security Proxy Core Type Definitions
 */
export interface JsonRpcRequest<T = unknown[]> {
    jsonrpc: "2.0";
    id: number | string;
    method: string;
    params?: T;
}
export interface JsonRpcSuccessResponse<T = unknown> {
    jsonrpc: "2.0";
    id: number | string;
    result: T;
}
export interface JsonRpcErrorObject {
    code: number;
    message: string;
    data?: {
        revert_data?: string;
        decoded_reason?: string;
        estimated_gas_saved?: number;
        simulation_latency_ms?: number;
    } | string;
}
export interface JsonRpcErrorResponse {
    jsonrpc: "2.0";
    id: number | string;
    error: JsonRpcErrorObject;
}
export type JsonRpcResponse<T = unknown> = JsonRpcSuccessResponse<T> | JsonRpcErrorResponse;
export interface SimulationResult {
    success: boolean;
    gasUsed?: bigint;
    gasSaved?: bigint;
    revertReason?: string;
    revertData?: string;
    latencyMs?: number;
}
export interface OpSecConfig {
    proxyUrl: string;
    fallbackRpcUrl?: string;
    timeoutMs?: number;
    autoDecodeReverts?: boolean;
}
export interface GasProtectionMetrics {
    totalSimulations: number;
    revertsBlocked: number;
    totalGasSavedWei: bigint;
    averageLatencyMs: number;
}
