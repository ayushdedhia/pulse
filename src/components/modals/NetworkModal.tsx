import { ArrowLeft, Copy, Monitor, RefreshCw, Server, Wifi, WifiOff } from "lucide-react";
import { useEffect, useState } from "react";

import { websocketService } from "../../services";
import { useUIStore } from "../../store/uiStore";
import type { NetworkStatus, PeerInfo } from "../../types";

export function NetworkModal() {
  const setShowNetwork = useUIStore((state) => state.setShowNetwork);
  const [networkStatus, setNetworkStatus] = useState<NetworkStatus | null>(null);
  const [peerIp, setPeerIp] = useState("");
  const [peerPort, setPeerPort] = useState("9001");
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [loading, setLoading] = useState(true);

  const loadNetworkStatus = async () => {
    try {
      const status = await websocketService.getNetworkStatus();
      setNetworkStatus(status);
    } catch (err) {
      console.error("Failed to load network status:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadNetworkStatus();
    // Refresh status every 5 seconds
    const interval = setInterval(loadNetworkStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleCopyIp = async () => {
    if (networkStatus?.local_ip) {
      const address = `${networkStatus.local_ip}:${networkStatus.port}`;
      await navigator.clipboard.writeText(address);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleConnect = async () => {
    if (!peerIp.trim()) {
      setError("Please enter an IP address");
      return;
    }

    setConnecting(true);
    setError(null);

    try {
      await websocketService.connectToPeer(peerIp.trim(), parseInt(peerPort) || 9001);
      setPeerIp("");
      // Refresh status after connecting
      setTimeout(loadNetworkStatus, 1000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to connect");
    } finally {
      setConnecting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50"
        onClick={() => setShowNetwork(false)}
      />

      {/* Modal Panel */}
      <div className="relative w-[350px] h-full bg-[var(--bg-primary)] flex flex-col shadow-2xl animate-slide-in">
        {/* Header */}
        <header className="flex items-center gap-6 px-4 py-4 bg-[var(--bg-secondary)]">
          <button
            onClick={() => setShowNetwork(false)}
            className="text-[var(--text-primary)] hover:text-[var(--text-secondary)] transition-colors"
          >
            <ArrowLeft size={24} />
          </button>
          <h2 className="text-lg font-medium text-[var(--text-primary)]">
            Network
          </h2>
          <button
            onClick={loadNetworkStatus}
            className="ml-auto text-[var(--text-secondary)] hover:text-[var(--accent)] transition-colors"
            title="Refresh"
          >
            <RefreshCw size={20} className={loading ? "animate-spin" : ""} />
          </button>
        </header>

        {/* Content */}
        <div className="flex-1 overflow-y-auto">
          {/* Status Section */}
          <div className="p-4 bg-[var(--bg-primary)]">
            <label className="text-sm text-[var(--accent)] mb-3 block">
              Connection Status
            </label>
            <div className="flex items-center gap-3 p-3 rounded-lg bg-[var(--bg-secondary)]">
              {networkStatus?.is_server ? (
                <>
                  <Server size={24} className="text-green-500" />
                  <div>
                    <p className="text-[var(--text-primary)] font-medium">Server Mode</p>
                    <p className="text-xs text-[var(--text-secondary)]">
                      Accepting connections from other devices
                    </p>
                  </div>
                </>
              ) : (
                <>
                  <Monitor size={24} className="text-blue-500" />
                  <div>
                    <p className="text-[var(--text-primary)] font-medium">Client Mode</p>
                    <p className="text-xs text-[var(--text-secondary)]">
                      Connected to another Pulse instance
                    </p>
                  </div>
                </>
              )}
            </div>
          </div>

          <div className="h-2 bg-[var(--bg-secondary)]" />

          {/* Your IP Section */}
          <div className="p-4 bg-[var(--bg-primary)]">
            <label className="text-sm text-[var(--accent)] mb-2 block">
              Your LAN Address
            </label>
            {networkStatus?.local_ip ? (
              <>
                <div className="flex items-center justify-between gap-2 p-3 rounded-lg bg-[var(--bg-secondary)]">
                  <span className="text-[var(--text-primary)] font-mono">
                    {networkStatus.local_ip}:{networkStatus.port}
                  </span>
                  <button
                    onClick={handleCopyIp}
                    className="flex items-center gap-1 px-2 py-1 text-xs text-[var(--accent)] hover:bg-[var(--bg-hover)] rounded transition-colors"
                  >
                    <Copy size={14} />
                    {copied ? "Copied!" : "Copy"}
                  </button>
                </div>
                <p className="text-xs text-[var(--text-secondary)] mt-2">
                  Share this address with others on your network to let them connect.
                </p>
              </>
            ) : (
              <p className="text-[var(--text-secondary)]">
                Could not detect local IP address
              </p>
            )}
          </div>

          <div className="h-2 bg-[var(--bg-secondary)]" />

          {/* Connect to Peer Section */}
          <div className="p-4 bg-[var(--bg-primary)]">
            <label className="text-sm text-[var(--accent)] mb-3 block">
              Connect to Peer
            </label>
            <div className="space-y-3">
              <div className="flex gap-2">
                <input
                  type="text"
                  value={peerIp}
                  onChange={(e) => setPeerIp(e.target.value)}
                  placeholder="IP Address (e.g., 192.168.1.100)"
                  className="flex-1 px-3 py-2 rounded-lg bg-[var(--bg-secondary)] text-[var(--text-primary)] placeholder-[var(--text-secondary)] outline-none focus:ring-2 focus:ring-[var(--accent)] font-mono"
                />
                <input
                  type="text"
                  value={peerPort}
                  onChange={(e) => setPeerPort(e.target.value)}
                  placeholder="Port"
                  className="w-20 px-3 py-2 rounded-lg bg-[var(--bg-secondary)] text-[var(--text-primary)] placeholder-[var(--text-secondary)] outline-none focus:ring-2 focus:ring-[var(--accent)] font-mono"
                />
              </div>
              <button
                onClick={handleConnect}
                disabled={connecting}
                className="w-full py-2 px-4 rounded-lg bg-[var(--accent)] text-white font-medium hover:bg-[var(--accent-dark)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {connecting ? "Connecting..." : "Connect"}
              </button>
              {error && (
                <p className="text-sm text-red-500">{error}</p>
              )}
            </div>
            <p className="text-xs text-[var(--text-secondary)] mt-3">
              Enter the LAN address of another Pulse user to connect directly.
            </p>
          </div>

          <div className="h-2 bg-[var(--bg-secondary)]" />

          {/* Connected Peers Section */}
          <div className="p-4 bg-[var(--bg-primary)]">
            <label className="text-sm text-[var(--accent)] mb-3 block">
              Connected Peers
            </label>
            {networkStatus?.connected_peers && networkStatus.connected_peers.length > 0 ? (
              <div className="space-y-2">
                {networkStatus.connected_peers.map((peer: PeerInfo, index: number) => (
                  <div
                    key={`${peer.ip}-${index}`}
                    className="flex items-center gap-3 p-3 rounded-lg bg-[var(--bg-secondary)]"
                  >
                    {peer.connected ? (
                      <Wifi size={20} className="text-green-500" />
                    ) : (
                      <WifiOff size={20} className="text-[var(--text-secondary)]" />
                    )}
                    <div className="flex-1">
                      <p className="text-[var(--text-primary)] font-mono text-sm">
                        {peer.ip}:{peer.port}
                      </p>
                      <p className="text-xs text-[var(--text-secondary)]">
                        {peer.connected ? "Connected" : "Disconnected"}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-[var(--text-secondary)] text-sm">
                No peers connected yet
              </p>
            )}
          </div>
        </div>
      </div>

      <style>{`
        @keyframes slide-in {
          from {
            transform: translateX(-100%);
          }
          to {
            transform: translateX(0);
          }
        }
        .animate-slide-in {
          animation: slide-in 0.2s ease-out;
        }
      `}</style>
    </div>
  );
}
