/**
 * OP Security Proxy Provider Client
 * Intercepts transactions and standardizes EIP-1193 request flows.
 */

import {
  JsonRpcRequest,
  JsonRpcResponse,
  OpSecConfig,
  GasProtectionMetrics,
  JsonRpcErrorResponse
} from "./types";
import { RevertBlockedError } from "./errors";

export class OpSecClient {
  private readonly config: OpSecConfig;
  private metrics: GasProtectionMetrics = {
    totalSimulations: 0,
    revertsBlocked: 0,
    totalGasSavedWei: BigInt(0),
    averageLatencyMs: 0
  };

  constructor(config: OpSecConfig) {
    this.config = {
      timeoutMs: 10000,
      autoDecodeReverts: true,
      ...config
    };
  }

  public getMetrics(): GasProtectionMetrics {
    return { ...this.metrics };
  }

  public async request<T = unknown>(req: JsonRpcRequest): Promise<T> {
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

      const json = (await response.json()) as JsonRpcResponse<T>;
      const elapsed = Date.now() - startTime;

      if ("error" in json) {
        const errResp = json as JsonRpcErrorResponse;
        if (isSendTx && this.config.autoDecodeReverts) {
          this.metrics.revertsBlocked += 1;
          this.metrics.totalSimulations += 1;
          
          if (typeof errResp.error.data === "object" && errResp.error.data?.estimated_gas_saved) {
            this.metrics.totalGasSavedWei += BigInt(errResp.error.data.estimated_gas_saved);
          }
          throw new RevertBlockedError(errResp.error);
        }
        throw new Error(`RPC Error [${errResp.error.code}]: ${errResp.error.message}`);
      }

      if (isSendTx) {
        this.metrics.totalSimulations += 1;
      }
      
      this.updateLatency(elapsed);
      return json.result;
    } catch (err) {
      if (err instanceof RevertBlockedError) {
        throw err;
      }
      if (this.config.fallbackRpcUrl && !isSendTx) {
        return this.fallbackRequest<T>(req);
      }
      throw err;
    }
  }

  private async fallbackRequest<T>(req: JsonRpcRequest): Promise<T> {
    const response = await fetch(this.config.fallbackRpcUrl!, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(req)
    });
    const json = (await response.json()) as JsonRpcResponse<T>;
    if ("error" in json) {
      throw new Error(`Fallback RPC Error: ${json.error.message}`);
    }
    return json.result;
  }

  private updateLatency(ms: number) {
    const n = this.metrics.totalSimulations || 1;
    this.metrics.averageLatencyMs = (this.metrics.averageLatencyMs * (n - 1) + ms) / n;
  }
}
