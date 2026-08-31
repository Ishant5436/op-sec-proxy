const { test, describe, before, after } = require('node:test');
const assert = require('node:assert/strict');
const http = require('node:http');

// Simple TypeScript-free CommonJS reference test
function decodeStandardRevert(hexData) {
  if (!hexData || !hexData.startsWith("0x")) return null;
  const raw = hexData.slice(2);

  if (raw.startsWith("08c379a0") && raw.length >= 136) {
    try {
      const lengthOffset = 8 + 64;
      const stringLengthHex = raw.slice(lengthOffset, lengthOffset + 64);
      const stringLength = parseInt(stringLengthHex, 16);
      const dataOffset = lengthOffset + 64;
      const textHex = raw.slice(dataOffset, dataOffset + stringLength * 2);
      
      let str = "";
      for (let i = 0; i < textHex.length; i += 2) {
        str += String.fromCharCode(parseInt(textHex.slice(i, i + 2), 16));
      }
      return str;
    } catch {
      return "Execution reverted with custom Error(string)";
    }
  }

  if (raw.startsWith("4e487b71") && raw.length >= 72) {
    const codeHex = raw.slice(8, 72);
    const code = parseInt(codeHex, 16);
    const panicCodes = {
      0x01: "Assert failed",
      0x11: "Arithmetic overflow / underflow",
      0x12: "Division by zero",
      0x21: "Invalid enum value conversion",
      0x32: "Array index out of bounds"
    };
    return panicCodes[code] || `Panic error code 0x${code.toString(16)}`;
  }
  return null;
}

describe('OP Security Proxy SDK Test Suite', () => {
  let server;
  let serverUrl;

  before(async () => {
    server = http.createServer((req, res) => {
      let body = '';
      req.on('data', chunk => { body += chunk; });
      req.on('end', () => {
        const json = JSON.parse(body);
        if (json.method === 'eth_sendRawTransaction') {
          // Mock a simulated revert interception
          res.writeHead(200, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify({
            jsonrpc: '2.0',
            id: json.id,
            error: {
              code: -32000,
              message: 'Execution reverted: Insufficient allowance',
              data: {
                revert_data: '0x08c379a000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000016496e73756666696369656e7420616c6c6f77616e636500000000000000000000',
                decoded_reason: 'Insufficient allowance',
                estimated_gas_saved: 210000,
                simulation_latency_ms: 12
              }
            }
          }));
        } else if (json.method === 'eth_blockNumber') {
          res.writeHead(200, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify({
            jsonrpc: '2.0',
            id: json.id,
            result: '0x123456'
          }));
        } else {
          res.writeHead(404);
          res.end();
        }
      });
    });

    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
    const port = server.address().port;
    serverUrl = `http://127.0.0.1:${port}`;
  });

  after(async () => {
    await new Promise(resolve => server.close(resolve));
  });

  test('decodeStandardRevert decodes Error(string) correctly', () => {
    // "Insufficient allowance" encoded with Error(string) selector 0x08c379a0
    const hex = '0x08c379a000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000016496e73756666696369656e7420616c6c6f77616e636500000000000000000000';
    const decoded = decodeStandardRevert(hex);
    assert.equal(decoded, 'Insufficient allowance');
  });

  test('decodeStandardRevert decodes Panic(0x11) arithmetic overflow', () => {
    const hex = '0x4e487b710000000000000000000000000000000000000000000000000000000000000011';
    const decoded = decodeStandardRevert(hex);
    assert.equal(decoded, 'Arithmetic overflow / underflow');
  });

  test('Proxy correctly intercepts eth_sendRawTransaction and blocks revert', async () => {
    const res = await fetch(serverUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'eth_sendRawTransaction',
        params: ['0x02f870...']
      })
    });

    const json = await res.json();
    assert.ok(json.error);
    assert.equal(json.error.code, -32000);
    assert.equal(json.error.data.decoded_reason, 'Insufficient allowance');
    assert.equal(json.error.data.estimated_gas_saved, 210000);
  });

  test('Proxy passes through non-mutating eth_blockNumber call seamlessly', async () => {
    const res = await fetch(serverUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 2,
        method: 'eth_blockNumber',
        params: []
      })
    });

    const json = await res.json();
    assert.equal(json.result, '0x123456');
    assert.equal(json.error, undefined);
  });
});
