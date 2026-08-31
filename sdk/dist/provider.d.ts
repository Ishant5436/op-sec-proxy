/**
 * OP Security Proxy Provider Client
 * Intercepts transactions and standardizes EIP-1193 request flows.
 */
import { JsonRpcRequest, OpSecConfig, GasProtectionMetrics } from "./types";
export declare class OpSecClient {
    private readonly config;
    private metrics;
    constructor(config: OpSecConfig);
    getMetrics(): GasProtectionMetrics;
    request<T = unknown>(req: JsonRpcRequest): Promise<T>;
    private fallbackRequest;
    private updateLatency;
}
