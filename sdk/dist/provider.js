"use strict";
/**
 * OP Security Proxy Provider Client
 * Intercepts transactions and standardizes EIP-1193 request flows.
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.OpSecClient = void 0;
const errors_1 = require("./errors");
class OpSecClient {
    config;
    metrics = {
        totalSimulations: 0,
        revertsBlocked: 0,
        totalGasSavedWei: BigInt(0),
        averageLatencyMs: 0
    };
    constructor(config) {
        this.config = {
            timeoutMs: 10000,
            autoDecodeReverts: true,
            ...config
        };
    }
    getMetrics() {
        return { ...this.metrics };
    }
    async request(req) {
        const startTime = Date.now();
        const isSendTx = req.method === "eth_sendRawTransaction" || req.method === "eth_sendTransaction";
        try {
            const response = await fetch(this.config.proxyUrl, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(req),
                signal: AbortSignal.timeout(this.config.timeoutMs || 10000)
            });
            if (!response.ok) {
                throw new Error(`HTTP error ${response.status}: ${response.statusText}`);
            }
            const json = (await response.json());
            const elapsed = Date.now() - startTime;
            if ("error" in json) {
                const errResp = json;
                if (isSendTx && this.config.autoDecodeReverts) {
                    this.metrics.revertsBlocked += 1;
                    this.metrics.totalSimulations += 1;
                    if (typeof errResp.error.data === "object" && errResp.error.data?.estimated_gas_saved) {
                        this.metrics.totalGasSavedWei += BigInt(errResp.error.data.estimated_gas_saved);
                    }
                    throw new errors_1.RevertBlockedError(errResp.error);
                }
                throw new Error(`RPC Error [${errResp.error.code}]: ${errResp.error.message}`);
            }
            if (isSendTx) {
                this.metrics.totalSimulations += 1;
            }
            this.updateLatency(elapsed);
            return json.result;
        }
        catch (err) {
            if (err instanceof errors_1.RevertBlockedError) {
                throw err;
            }
            if (this.config.fallbackRpcUrl && !isSendTx) {
                return this.fallbackRequest(req);
            }
            throw err;
        }
    }
    async fallbackRequest(req) {
        const response = await fetch(this.config.fallbackRpcUrl, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(req)
        });
        const json = (await response.json());
        if ("error" in json) {
            throw new Error(`Fallback RPC Error: ${json.error.message}`);
        }
        return json.result;
    }
    updateLatency(ms) {
        const n = this.metrics.totalSimulations || 1;
        this.metrics.averageLatencyMs = (this.metrics.averageLatencyMs * (n - 1) + ms) / n;
    }
}
exports.OpSecClient = OpSecClient;
