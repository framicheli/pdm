// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use config::{Config, File, FileFormat};
use std::{collections::HashSet, path::Path};

/// Core Config
#[derive(Debug, Clone)]
pub struct Core {
    // Data directory and storage
    datadir: Option<String>,
    blocksdir: Option<String>,
    pid: Option<String>,
    debuglogfile: Option<String>,
    settings: Option<String>,
    includeconf: Option<String>,
    loadblock: Option<String>,

    // Indexing
    txindex: Option<bool>,
    blockfilterindex: Option<String>,
    coinstatsindex: Option<bool>,

    // Pruning
    prune: Option<u32>,

    // Memory and performance
    dbcache: Option<u32>,
    maxmempool: Option<u32>,
    maxorphantx: Option<u32>,
    mempoolexpiry: Option<u32>,
    par: Option<i32>,
    blockreconstructionextratxn: Option<u32>,

    // Behavior
    blocksonly: Option<bool>,
    persistmempool: Option<bool>,
    reindex: Option<bool>,
    reindex_chainstate: Option<bool>,
    sysperms: Option<bool>,

    // Daemon mode
    daemon: Option<bool>,
    daemonwait: Option<bool>,

    // Notification commands
    alertnotify: Option<String>,
    blocknotify: Option<String>,
    startupnotify: Option<String>,

    // Validation
    assumevalid: Option<String>,
}

/// Network Config
#[derive(Debug, Clone)]
pub struct Network {
    // Chain selection
    chain: Option<String>,
    testnet: Option<bool>,
    regtest: Option<bool>,
    signet: Option<bool>,
    signetchallenge: Option<String>,
    signetseednode: Option<String>,

    // Listening and binding
    listen: Option<bool>,
    bind: Option<String>,
    whitebind: Option<String>,
    port: Option<u32>,

    // Connection limits
    maxconnections: Option<u32>,
    maxreceivebuffer: Option<u32>,
    maxsendbuffer: Option<u32>,
    maxuploadtarget: Option<u32>,
    timeout: Option<u32>,
    maxtimeadjustment: Option<u32>,
    bantime: Option<u32>,

    // Peer discovery
    discover: Option<bool>,
    dns: Option<bool>,
    dnsseed: Option<bool>,
    fixedseeds: Option<bool>,
    forcednsseed: Option<bool>,
    seednode: Option<String>,
    addnode: Option<String>,
    connect: Option<String>,

    // Network selection
    onlynet: Option<String>,
    networkactive: Option<bool>,

    // Proxy settings
    proxy: Option<String>,
    proxyrandomize: Option<bool>,

    // Tor settings
    onion: Option<String>,
    listenonion: Option<bool>,
    torcontrol: Option<String>,
    torpassword: Option<String>,

    // I2P settings
    i2psam: Option<String>,
    i2pacceptincoming: Option<bool>,

    // CJDNS
    cjdnsreachable: Option<bool>,

    // Peer permissions
    whitelist: Option<String>,
    peerblockfilters: Option<bool>,
    peerbloomfilters: Option<bool>,
    permitbaremultisig: Option<bool>,

    // External IP
    externalip: Option<String>,

    // UPnP
    upnp: Option<bool>,

    // ASN mapping
    asmap: Option<String>,
}

/// RPC Config
#[derive(Debug, Clone)]
pub struct RPC {
    // Server enable
    server: Option<bool>,

    // Authentication
    rpcuser: Option<String>,
    rpcpassword: Option<String>,
    rpcauth: Option<String>,
    rpccookiefile: Option<String>,

    // Connection
    rpcport: Option<u32>,
    rpcbind: Option<String>,
    rpcallowip: Option<String>,

    // Performance
    rpcthreads: Option<u32>,

    // Serialization
    rpcserialversion: Option<u32>,

    // Whitelist
    rpcwhitelist: Option<String>,
    rpcwhitelistdefault: Option<bool>,

    // REST interface
    rest: Option<bool>,
}

/// Wallet related config
#[derive(Debug, Clone)]
pub struct Wallet {
    // Enable/disable
    disablewallet: Option<bool>,

    // Wallet paths
    wallet: Option<String>,
    walletdir: Option<String>,

    // Address types
    addresstype: Option<String>,
    changetype: Option<String>,

    // Fee settings
    fallbackfee: Option<String>,
    discardfee: Option<String>,
    mintxfee: Option<String>,
    paytxfee: Option<String>,
    consolidatefeerate: Option<String>,
    maxapsfee: Option<String>,

    // Transaction behavior
    txconfirmtarget: Option<u32>,
    spendzeroconfchange: Option<bool>,
    walletrbf: Option<bool>,
    avoidpartialspends: Option<bool>,

    // Key management
    keypool: Option<u32>,

    // External signer
    signer: Option<String>,

    // Broadcast
    walletbroadcast: Option<bool>,

    // Notifications
    walletnotify: Option<String>,
}

/// Debugging related config
#[derive(Debug, Clone)]
pub struct Debugging {
    // Debug categories
    debug: Option<String>,
    debugexclude: Option<String>,

    // Logging options
    logips: Option<bool>,
    logsourcelocations: Option<bool>,
    logthreadnames: Option<bool>,
    logtimestamps: Option<bool>,
    shrinkdebugfile: Option<bool>,
    printtoconsole: Option<bool>,

    // User agent
    uacomment: Option<String>,

    // Fee limits
    maxtxfee: Option<String>,
}

/// Mining related config
#[derive(Debug, Clone)]
pub struct Mining {
    // Block creation
    blockmaxweight: Option<u32>,
    blockmintxfee: Option<String>,
}

/// Relay related config
#[derive(Debug, Clone)]
pub struct Relay {
    // Relay fees
    minrelaytxfee: Option<String>,

    // Data carrier (OP_RETURN)
    datacarrier: Option<bool>,
    datacarriersize: Option<u32>,

    // Sigops
    bytespersigop: Option<u32>,

    // Whitelist relay
    whitelistforcerelay: Option<bool>,
    whitelistrelay: Option<bool>,
}

/// ZMQ related config
#[derive(Debug, Clone)]
pub struct ZMQ {
    // Hash notifications
    zmqpubhashblock: Option<String>,
    zmqpubhashtx: Option<String>,

    // Raw data notifications
    zmqpubrawblock: Option<String>,
    zmqpubrawtx: Option<String>,

    // Sequence notifications
    zmqpubsequence: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BitcoinConfig {
    core: Core,
    network: Network,
    rpc: RPC,
    wallet: Wallet,
    debugging: Debugging,
    mining: Mining,
    relay: Relay,
    zmq: ZMQ,
}

/// Type of a configuration option value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigType {
    Bool,
    Int,
    Float,
    String,
    Path,
    Address,
}

/// Category of a configuration option
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigCategory {
    Core,
    Network,
    RPC,
    Wallet,
    Debugging,
    Mining,
    Relay,
    ZMQ,
}

/// Schema for a single configuration option
#[derive(Debug, Clone)]
pub struct ConfigSchema {
    pub key: String,
    pub default: String,
    pub config_type: ConfigType,
    pub category: ConfigCategory,
    pub description: String,
}

impl ConfigSchema {
    pub fn new(
        key: &str,
        default: &str,
        config_type: ConfigType,
        category: ConfigCategory,
        description: &str,
    ) -> Self {
        Self {
            key: key.to_string(),
            default: default.to_string(),
            config_type,
            category,
            description: description.to_string(),
        }
    }
}

/// A parsed configuration entry
#[derive(Debug, Clone)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub schema: Option<ConfigSchema>,
    pub enabled: bool,
}

/// Returns the default schema for all known bitcoin.conf options
pub fn get_default_schema() -> Vec<ConfigSchema> {
    vec![
        // Core options
        ConfigSchema::new(
            "datadir",
            "",
            ConfigType::Path,
            ConfigCategory::Core,
            "Specify data directory",
        ),
        ConfigSchema::new(
            "blocksdir",
            "",
            ConfigType::Path,
            ConfigCategory::Core,
            "Specify blocks directory",
        ),
        ConfigSchema::new(
            "pid",
            "",
            ConfigType::Path,
            ConfigCategory::Core,
            "Specify pid file",
        ),
        ConfigSchema::new(
            "debuglogfile",
            "",
            ConfigType::Path,
            ConfigCategory::Core,
            "Specify debug log file",
        ),
        ConfigSchema::new(
            "settings",
            "",
            ConfigType::Path,
            ConfigCategory::Core,
            "Specify settings file",
        ),
        ConfigSchema::new(
            "includeconf",
            "",
            ConfigType::Path,
            ConfigCategory::Core,
            "Include additional config file",
        ),
        ConfigSchema::new(
            "loadblock",
            "",
            ConfigType::Path,
            ConfigCategory::Core,
            "Import blocks from external file",
        ),
        ConfigSchema::new(
            "txindex",
            "0",
            ConfigType::Bool,
            ConfigCategory::Core,
            "Maintain full transaction index",
        ),
        ConfigSchema::new(
            "blockfilterindex",
            "",
            ConfigType::String,
            ConfigCategory::Core,
            "Maintain compact block filter index",
        ),
        ConfigSchema::new(
            "coinstatsindex",
            "0",
            ConfigType::Bool,
            ConfigCategory::Core,
            "Maintain coinstats index",
        ),
        ConfigSchema::new(
            "prune",
            "0",
            ConfigType::Int,
            ConfigCategory::Core,
            "Reduce storage by pruning old blocks",
        ),
        ConfigSchema::new(
            "dbcache",
            "450",
            ConfigType::Int,
            ConfigCategory::Core,
            "Database cache size in MiB",
        ),
        ConfigSchema::new(
            "maxmempool",
            "300",
            ConfigType::Int,
            ConfigCategory::Core,
            "Maximum mempool size in MiB",
        ),
        ConfigSchema::new(
            "maxorphantx",
            "100",
            ConfigType::Int,
            ConfigCategory::Core,
            "Maximum orphan transactions",
        ),
        ConfigSchema::new(
            "mempoolexpiry",
            "336",
            ConfigType::Int,
            ConfigCategory::Core,
            "Mempool expiry in hours",
        ),
        ConfigSchema::new(
            "par",
            "0",
            ConfigType::Int,
            ConfigCategory::Core,
            "Script verification threads",
        ),
        ConfigSchema::new(
            "blockreconstructionextratxn",
            "100",
            ConfigType::Int,
            ConfigCategory::Core,
            "Extra transactions for block reconstruction",
        ),
        ConfigSchema::new(
            "blocksonly",
            "0",
            ConfigType::Bool,
            ConfigCategory::Core,
            "Reject transactions from network peers",
        ),
        ConfigSchema::new(
            "persistmempool",
            "1",
            ConfigType::Bool,
            ConfigCategory::Core,
            "Save mempool on shutdown",
        ),
        ConfigSchema::new(
            "reindex",
            "0",
            ConfigType::Bool,
            ConfigCategory::Core,
            "Rebuild chain state and block index",
        ),
        ConfigSchema::new(
            "reindex-chainstate",
            "0",
            ConfigType::Bool,
            ConfigCategory::Core,
            "Rebuild chain state from blocks",
        ),
        ConfigSchema::new(
            "sysperms",
            "0",
            ConfigType::Bool,
            ConfigCategory::Core,
            "Create files with system default permissions",
        ),
        ConfigSchema::new(
            "daemon",
            "0",
            ConfigType::Bool,
            ConfigCategory::Core,
            "Run in background as daemon",
        ),
        ConfigSchema::new(
            "daemonwait",
            "0",
            ConfigType::Bool,
            ConfigCategory::Core,
            "Wait for initialization before backgrounding",
        ),
        ConfigSchema::new(
            "alertnotify",
            "",
            ConfigType::String,
            ConfigCategory::Core,
            "Command to execute on alert",
        ),
        ConfigSchema::new(
            "blocknotify",
            "",
            ConfigType::String,
            ConfigCategory::Core,
            "Command to execute on new block",
        ),
        ConfigSchema::new(
            "startupnotify",
            "",
            ConfigType::String,
            ConfigCategory::Core,
            "Command to execute on startup",
        ),
        ConfigSchema::new(
            "assumevalid",
            "",
            ConfigType::String,
            ConfigCategory::Core,
            "Assume blocks are valid up to this hash",
        ),
        // Network options
        ConfigSchema::new(
            "chain",
            "main",
            ConfigType::String,
            ConfigCategory::Network,
            "Chain to use (main, test, signet, regtest)",
        ),
        ConfigSchema::new(
            "testnet",
            "0",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Use testnet",
        ),
        ConfigSchema::new(
            "regtest",
            "0",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Use regtest",
        ),
        ConfigSchema::new(
            "signet",
            "0",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Use signet",
        ),
        ConfigSchema::new(
            "signetchallenge",
            "",
            ConfigType::String,
            ConfigCategory::Network,
            "Signet challenge script",
        ),
        ConfigSchema::new(
            "signetseednode",
            "",
            ConfigType::String,
            ConfigCategory::Network,
            "Signet seed node",
        ),
        ConfigSchema::new(
            "listen",
            "1",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Accept incoming connections",
        ),
        ConfigSchema::new(
            "bind",
            "",
            ConfigType::Address,
            ConfigCategory::Network,
            "Bind to address",
        ),
        ConfigSchema::new(
            "whitebind",
            "",
            ConfigType::Address,
            ConfigCategory::Network,
            "Bind with whitelist permissions",
        ),
        ConfigSchema::new(
            "port",
            "8333",
            ConfigType::Int,
            ConfigCategory::Network,
            "Listen on port",
        ),
        ConfigSchema::new(
            "maxconnections",
            "125",
            ConfigType::Int,
            ConfigCategory::Network,
            "Maximum peer connections",
        ),
        ConfigSchema::new(
            "maxreceivebuffer",
            "5000",
            ConfigType::Int,
            ConfigCategory::Network,
            "Maximum receive buffer per connection",
        ),
        ConfigSchema::new(
            "maxsendbuffer",
            "1000",
            ConfigType::Int,
            ConfigCategory::Network,
            "Maximum send buffer per connection",
        ),
        ConfigSchema::new(
            "maxuploadtarget",
            "0",
            ConfigType::Int,
            ConfigCategory::Network,
            "Maximum upload target in MiB per day",
        ),
        ConfigSchema::new(
            "timeout",
            "5000",
            ConfigType::Int,
            ConfigCategory::Network,
            "Connection timeout in milliseconds",
        ),
        ConfigSchema::new(
            "maxtimeadjustment",
            "4200",
            ConfigType::Int,
            ConfigCategory::Network,
            "Maximum time adjustment in seconds",
        ),
        ConfigSchema::new(
            "bantime",
            "86400",
            ConfigType::Int,
            ConfigCategory::Network,
            "Ban duration in seconds",
        ),
        ConfigSchema::new(
            "discover",
            "1",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Discover own IP address",
        ),
        ConfigSchema::new(
            "dns",
            "1",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Allow DNS lookups",
        ),
        ConfigSchema::new(
            "dnsseed",
            "1",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Query DNS seeds",
        ),
        ConfigSchema::new(
            "fixedseeds",
            "1",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Use fixed seeds if DNS fails",
        ),
        ConfigSchema::new(
            "forcednsseed",
            "0",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Always query DNS seeds",
        ),
        ConfigSchema::new(
            "seednode",
            "",
            ConfigType::Address,
            ConfigCategory::Network,
            "Connect to seed node for addresses",
        ),
        ConfigSchema::new(
            "addnode",
            "",
            ConfigType::Address,
            ConfigCategory::Network,
            "Add node to connect to",
        ),
        ConfigSchema::new(
            "connect",
            "",
            ConfigType::Address,
            ConfigCategory::Network,
            "Connect only to specified node",
        ),
        ConfigSchema::new(
            "onlynet",
            "",
            ConfigType::String,
            ConfigCategory::Network,
            "Only connect to network type",
        ),
        ConfigSchema::new(
            "networkactive",
            "1",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Enable network activity",
        ),
        ConfigSchema::new(
            "proxy",
            "",
            ConfigType::Address,
            ConfigCategory::Network,
            "SOCKS5 proxy",
        ),
        ConfigSchema::new(
            "proxyrandomize",
            "1",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Randomize proxy credentials",
        ),
        ConfigSchema::new(
            "onion",
            "",
            ConfigType::Address,
            ConfigCategory::Network,
            "SOCKS5 proxy for Tor",
        ),
        ConfigSchema::new(
            "listenonion",
            "1",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Create Tor onion service",
        ),
        ConfigSchema::new(
            "torcontrol",
            "127.0.0.1:9051",
            ConfigType::Address,
            ConfigCategory::Network,
            "Tor control port",
        ),
        ConfigSchema::new(
            "torpassword",
            "",
            ConfigType::String,
            ConfigCategory::Network,
            "Tor control password",
        ),
        ConfigSchema::new(
            "i2psam",
            "",
            ConfigType::Address,
            ConfigCategory::Network,
            "I2P SAM proxy",
        ),
        ConfigSchema::new(
            "i2pacceptincoming",
            "1",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Accept incoming I2P connections",
        ),
        ConfigSchema::new(
            "cjdnsreachable",
            "0",
            ConfigType::Bool,
            ConfigCategory::Network,
            "CJDNS reachable",
        ),
        ConfigSchema::new(
            "whitelist",
            "",
            ConfigType::String,
            ConfigCategory::Network,
            "Whitelist peers",
        ),
        ConfigSchema::new(
            "peerblockfilters",
            "0",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Serve compact block filters",
        ),
        ConfigSchema::new(
            "peerbloomfilters",
            "0",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Support bloom filters",
        ),
        ConfigSchema::new(
            "permitbaremultisig",
            "1",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Relay bare multisig",
        ),
        ConfigSchema::new(
            "externalip",
            "",
            ConfigType::Address,
            ConfigCategory::Network,
            "Specify external IP",
        ),
        ConfigSchema::new(
            "upnp",
            "0",
            ConfigType::Bool,
            ConfigCategory::Network,
            "Use UPnP for port mapping",
        ),
        ConfigSchema::new(
            "asmap",
            "",
            ConfigType::Path,
            ConfigCategory::Network,
            "ASN mapping file",
        ),
        // RPC options
        ConfigSchema::new(
            "server",
            "0",
            ConfigType::Bool,
            ConfigCategory::RPC,
            "Accept RPC commands",
        ),
        ConfigSchema::new(
            "rpcuser",
            "",
            ConfigType::String,
            ConfigCategory::RPC,
            "RPC username",
        ),
        ConfigSchema::new(
            "rpcpassword",
            "",
            ConfigType::String,
            ConfigCategory::RPC,
            "RPC password",
        ),
        ConfigSchema::new(
            "rpcauth",
            "",
            ConfigType::String,
            ConfigCategory::RPC,
            "RPC auth credentials",
        ),
        ConfigSchema::new(
            "rpccookiefile",
            "",
            ConfigType::Path,
            ConfigCategory::RPC,
            "RPC cookie file location",
        ),
        ConfigSchema::new(
            "rpcport",
            "8332",
            ConfigType::Int,
            ConfigCategory::RPC,
            "RPC port",
        ),
        ConfigSchema::new(
            "rpcbind",
            "",
            ConfigType::Address,
            ConfigCategory::RPC,
            "RPC bind address",
        ),
        ConfigSchema::new(
            "rpcallowip",
            "",
            ConfigType::String,
            ConfigCategory::RPC,
            "Allow RPC from IP",
        ),
        ConfigSchema::new(
            "rpcthreads",
            "4",
            ConfigType::Int,
            ConfigCategory::RPC,
            "RPC worker threads",
        ),
        ConfigSchema::new(
            "rpcserialversion",
            "1",
            ConfigType::Int,
            ConfigCategory::RPC,
            "RPC serialization version",
        ),
        ConfigSchema::new(
            "rpcwhitelist",
            "",
            ConfigType::String,
            ConfigCategory::RPC,
            "RPC method whitelist",
        ),
        ConfigSchema::new(
            "rpcwhitelistdefault",
            "1",
            ConfigType::Bool,
            ConfigCategory::RPC,
            "Default RPC whitelist behavior",
        ),
        ConfigSchema::new(
            "rest",
            "0",
            ConfigType::Bool,
            ConfigCategory::RPC,
            "Enable REST interface",
        ),
        // Wallet options
        ConfigSchema::new(
            "disablewallet",
            "0",
            ConfigType::Bool,
            ConfigCategory::Wallet,
            "Disable wallet",
        ),
        ConfigSchema::new(
            "wallet",
            "",
            ConfigType::Path,
            ConfigCategory::Wallet,
            "Wallet to load",
        ),
        ConfigSchema::new(
            "walletdir",
            "",
            ConfigType::Path,
            ConfigCategory::Wallet,
            "Wallet directory",
        ),
        ConfigSchema::new(
            "addresstype",
            "bech32",
            ConfigType::String,
            ConfigCategory::Wallet,
            "Default address type",
        ),
        ConfigSchema::new(
            "changetype",
            "",
            ConfigType::String,
            ConfigCategory::Wallet,
            "Change address type",
        ),
        ConfigSchema::new(
            "fallbackfee",
            "0.00",
            ConfigType::Float,
            ConfigCategory::Wallet,
            "Fallback fee rate",
        ),
        ConfigSchema::new(
            "discardfee",
            "0.0001",
            ConfigType::Float,
            ConfigCategory::Wallet,
            "Discard fee threshold",
        ),
        ConfigSchema::new(
            "mintxfee",
            "0.00001",
            ConfigType::Float,
            ConfigCategory::Wallet,
            "Minimum transaction fee",
        ),
        ConfigSchema::new(
            "paytxfee",
            "0.00",
            ConfigType::Float,
            ConfigCategory::Wallet,
            "Transaction fee rate",
        ),
        ConfigSchema::new(
            "consolidatefeerate",
            "0.0001",
            ConfigType::Float,
            ConfigCategory::Wallet,
            "Consolidation fee rate",
        ),
        ConfigSchema::new(
            "maxapsfee",
            "0.00",
            ConfigType::Float,
            ConfigCategory::Wallet,
            "Max fee for partial spend avoidance",
        ),
        ConfigSchema::new(
            "txconfirmtarget",
            "6",
            ConfigType::Int,
            ConfigCategory::Wallet,
            "Confirmation target blocks",
        ),
        ConfigSchema::new(
            "spendzeroconfchange",
            "1",
            ConfigType::Bool,
            ConfigCategory::Wallet,
            "Spend unconfirmed change",
        ),
        ConfigSchema::new(
            "walletrbf",
            "0",
            ConfigType::Bool,
            ConfigCategory::Wallet,
            "Enable wallet RBF",
        ),
        ConfigSchema::new(
            "avoidpartialspends",
            "0",
            ConfigType::Bool,
            ConfigCategory::Wallet,
            "Avoid partial spends",
        ),
        ConfigSchema::new(
            "keypool",
            "1000",
            ConfigType::Int,
            ConfigCategory::Wallet,
            "Keypool size",
        ),
        ConfigSchema::new(
            "signer",
            "",
            ConfigType::String,
            ConfigCategory::Wallet,
            "External signer command",
        ),
        ConfigSchema::new(
            "walletbroadcast",
            "1",
            ConfigType::Bool,
            ConfigCategory::Wallet,
            "Broadcast wallet transactions",
        ),
        ConfigSchema::new(
            "walletnotify",
            "",
            ConfigType::String,
            ConfigCategory::Wallet,
            "Command on wallet transaction",
        ),
        // Debugging options
        ConfigSchema::new(
            "debug",
            "",
            ConfigType::String,
            ConfigCategory::Debugging,
            "Debug categories",
        ),
        ConfigSchema::new(
            "debugexclude",
            "",
            ConfigType::String,
            ConfigCategory::Debugging,
            "Exclude debug categories",
        ),
        ConfigSchema::new(
            "logips",
            "0",
            ConfigType::Bool,
            ConfigCategory::Debugging,
            "Log IP addresses",
        ),
        ConfigSchema::new(
            "logsourcelocations",
            "0",
            ConfigType::Bool,
            ConfigCategory::Debugging,
            "Log source locations",
        ),
        ConfigSchema::new(
            "logthreadnames",
            "0",
            ConfigType::Bool,
            ConfigCategory::Debugging,
            "Log thread names",
        ),
        ConfigSchema::new(
            "logtimestamps",
            "1",
            ConfigType::Bool,
            ConfigCategory::Debugging,
            "Log timestamps",
        ),
        ConfigSchema::new(
            "shrinkdebugfile",
            "1",
            ConfigType::Bool,
            ConfigCategory::Debugging,
            "Shrink debug.log on startup",
        ),
        ConfigSchema::new(
            "printtoconsole",
            "0",
            ConfigType::Bool,
            ConfigCategory::Debugging,
            "Print to console",
        ),
        ConfigSchema::new(
            "uacomment",
            "",
            ConfigType::String,
            ConfigCategory::Debugging,
            "User agent comment",
        ),
        ConfigSchema::new(
            "maxtxfee",
            "0.10",
            ConfigType::Float,
            ConfigCategory::Debugging,
            "Maximum transaction fee",
        ),
        // Mining options
        ConfigSchema::new(
            "blockmaxweight",
            "3996000",
            ConfigType::Int,
            ConfigCategory::Mining,
            "Maximum block weight",
        ),
        ConfigSchema::new(
            "blockmintxfee",
            "0.00001",
            ConfigType::Float,
            ConfigCategory::Mining,
            "Minimum block transaction fee",
        ),
        // Relay options
        ConfigSchema::new(
            "minrelaytxfee",
            "0.00001",
            ConfigType::Float,
            ConfigCategory::Relay,
            "Minimum relay fee",
        ),
        ConfigSchema::new(
            "datacarrier",
            "1",
            ConfigType::Bool,
            ConfigCategory::Relay,
            "Relay OP_RETURN transactions",
        ),
        ConfigSchema::new(
            "datacarriersize",
            "83",
            ConfigType::Int,
            ConfigCategory::Relay,
            "Maximum OP_RETURN size",
        ),
        ConfigSchema::new(
            "bytespersigop",
            "20",
            ConfigType::Int,
            ConfigCategory::Relay,
            "Bytes per sigop",
        ),
        ConfigSchema::new(
            "whitelistforcerelay",
            "0",
            ConfigType::Bool,
            ConfigCategory::Relay,
            "Force relay from whitelist",
        ),
        ConfigSchema::new(
            "whitelistrelay",
            "1",
            ConfigType::Bool,
            ConfigCategory::Relay,
            "Relay from whitelist",
        ),
        // ZMQ options
        ConfigSchema::new(
            "zmqpubhashblock",
            "",
            ConfigType::Address,
            ConfigCategory::ZMQ,
            "ZMQ hash block publisher",
        ),
        ConfigSchema::new(
            "zmqpubhashtx",
            "",
            ConfigType::Address,
            ConfigCategory::ZMQ,
            "ZMQ hash tx publisher",
        ),
        ConfigSchema::new(
            "zmqpubrawblock",
            "",
            ConfigType::Address,
            ConfigCategory::ZMQ,
            "ZMQ raw block publisher",
        ),
        ConfigSchema::new(
            "zmqpubrawtx",
            "",
            ConfigType::Address,
            ConfigCategory::ZMQ,
            "ZMQ raw tx publisher",
        ),
        ConfigSchema::new(
            "zmqpubsequence",
            "",
            ConfigType::Address,
            ConfigCategory::ZMQ,
            "ZMQ sequence publisher",
        ),
    ]
}

/// Parse bitcoin.conf file
pub fn parse_config(path: &Path) -> Result<Vec<ConfigEntry>> {
    let schema_list = get_default_schema();
    let mut entries = Vec::new();
    let mut found_keys: HashSet<String> = HashSet::new();
    let mut builder = Config::builder();

    if path.exists() {
        builder = builder.add_source(File::from(path).format(FileFormat::Ini));
    }

    let config = match builder.build() {
        Ok(cfg) => cfg,
        Err(_) => {
            // Return schema defaults if config can't be parsed
            for schema in schema_list {
                entries.push(ConfigEntry {
                    key: schema.key.clone(),
                    value: schema.default.clone(),
                    schema: Some(schema),
                    enabled: false,
                });
            }
            return Ok(entries);
        }
    };

    let mut config_keys: HashSet<String> = HashSet::new();
    let sections = vec!["", "main", "test", "signet", "regtest"];

    // Collect all keys from all sections
    for section in &sections {
        if let Ok(table) = if section.is_empty() {
            config.get_table("")
        } else {
            config.get_table(section)
        } {
            for key in table.keys() {
                let actual_key = if key.contains('.') {
                    key.split('.').next_back().unwrap_or(key).to_string()
                } else {
                    key.clone()
                };
                config_keys.insert(actual_key);
            }
        }
    }

    // Process known schema options
    for schema in &schema_list {
        let key = &schema.key;
        let mut value = schema.default.clone();
        let mut enabled = false;

        for section in &sections {
            let lookup_key = if section.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", section, key)
            };

            if let Ok(val) = config.get_string(&lookup_key) {
                value = val;
                enabled = true;
                found_keys.insert(key.clone());
                break;
            }

            if let Ok(val) = config.get_bool(&lookup_key) {
                value = if val {
                    "1".to_string()
                } else {
                    "0".to_string()
                };
                enabled = true;
                found_keys.insert(key.clone());
                break;
            }

            if let Ok(val) = config.get_int(&lookup_key) {
                value = val.to_string();
                enabled = true;
                found_keys.insert(key.clone());
                break;
            }

            if let Ok(val) = config.get_float(&lookup_key) {
                value = val.to_string();
                enabled = true;
                found_keys.insert(key.clone());
                break;
            }
        }

        entries.push(ConfigEntry {
            key: key.clone(),
            value,
            schema: Some(schema.clone()),
            enabled,
        });
    }

    // Add unknown config keys (not in schema)
    for config_key in &config_keys {
        if !found_keys.contains(config_key) {
            // Try to get value from various sections
            let mut value = String::new();
            for section in &sections {
                let lookup_key = if section.is_empty() {
                    config_key.clone()
                } else {
                    format!("{}.{}", section, config_key)
                };

                if let Ok(val) = config.get_string(&lookup_key) {
                    value = val;
                    break;
                }
                if let Ok(val) = config.get_bool(&lookup_key) {
                    value = if val {
                        "1".to_string()
                    } else {
                        "0".to_string()
                    };
                    break;
                }
                if let Ok(val) = config.get_int(&lookup_key) {
                    value = val.to_string();
                    break;
                }
                if let Ok(val) = config.get_float(&lookup_key) {
                    value = val.to_string();
                    break;
                }
            }

            entries.push(ConfigEntry {
                key: config_key.clone(),
                value,
                schema: None,
                enabled: true,
            });
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_temp_config(content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("bitcoin.conf");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        (dir, file_path)
    }

    // Tests for get_default_schema()

    #[test]
    fn get_default_schema_returns_non_empty_list() {
        let schema = get_default_schema();
        assert!(!schema.is_empty());
    }

    #[test]
    fn get_default_schema_contains_core_options() {
        let schema = get_default_schema();
        let keys: Vec<&str> = schema.iter().map(|s| s.key.as_str()).collect();

        assert!(keys.contains(&"datadir"));
        assert!(keys.contains(&"txindex"));
        assert!(keys.contains(&"prune"));
        assert!(keys.contains(&"dbcache"));
    }

    #[test]
    fn get_default_schema_contains_network_options() {
        let schema = get_default_schema();
        let keys: Vec<&str> = schema.iter().map(|s| s.key.as_str()).collect();

        assert!(keys.contains(&"testnet"));
        assert!(keys.contains(&"regtest"));
        assert!(keys.contains(&"listen"));
        assert!(keys.contains(&"port"));
        assert!(keys.contains(&"maxconnections"));
    }

    #[test]
    fn get_default_schema_contains_rpc_options() {
        let schema = get_default_schema();
        let keys: Vec<&str> = schema.iter().map(|s| s.key.as_str()).collect();

        assert!(keys.contains(&"server"));
        assert!(keys.contains(&"rpcuser"));
        assert!(keys.contains(&"rpcpassword"));
        assert!(keys.contains(&"rpcport"));
    }

    #[test]
    fn get_default_schema_contains_zmq_options() {
        let schema = get_default_schema();
        let keys: Vec<&str> = schema.iter().map(|s| s.key.as_str()).collect();

        assert!(keys.contains(&"zmqpubhashblock"));
        assert!(keys.contains(&"zmqpubhashtx"));
        assert!(keys.contains(&"zmqpubrawblock"));
        assert!(keys.contains(&"zmqpubrawtx"));
        assert!(keys.contains(&"zmqpubsequence"));
    }

    #[test]
    fn get_default_schema_has_correct_categories() {
        let schema = get_default_schema();

        let txindex = schema.iter().find(|s| s.key == "txindex").unwrap();
        assert_eq!(txindex.category, ConfigCategory::Core);

        let testnet = schema.iter().find(|s| s.key == "testnet").unwrap();
        assert_eq!(testnet.category, ConfigCategory::Network);

        let server = schema.iter().find(|s| s.key == "server").unwrap();
        assert_eq!(server.category, ConfigCategory::RPC);

        let disablewallet = schema.iter().find(|s| s.key == "disablewallet").unwrap();
        assert_eq!(disablewallet.category, ConfigCategory::Wallet);
    }

    #[test]
    fn get_default_schema_has_correct_types() {
        let schema = get_default_schema();

        let txindex = schema.iter().find(|s| s.key == "txindex").unwrap();
        assert_eq!(txindex.config_type, ConfigType::Bool);

        let dbcache = schema.iter().find(|s| s.key == "dbcache").unwrap();
        assert_eq!(dbcache.config_type, ConfigType::Int);

        let fallbackfee = schema.iter().find(|s| s.key == "fallbackfee").unwrap();
        assert_eq!(fallbackfee.config_type, ConfigType::Float);

        let datadir = schema.iter().find(|s| s.key == "datadir").unwrap();
        assert_eq!(datadir.config_type, ConfigType::Path);

        let rpcbind = schema.iter().find(|s| s.key == "rpcbind").unwrap();
        assert_eq!(rpcbind.config_type, ConfigType::Address);
    }

    // Tests for ConfigSchema::new()

    #[test]
    fn config_schema_new_creates_correct_schema() {
        let schema = ConfigSchema::new(
            "testkey",
            "testdefault",
            ConfigType::String,
            ConfigCategory::Core,
            "Test description",
        );

        assert_eq!(schema.key, "testkey");
        assert_eq!(schema.default, "testdefault");
        assert_eq!(schema.config_type, ConfigType::String);
        assert_eq!(schema.category, ConfigCategory::Core);
        assert_eq!(schema.description, "Test description");
    }

    // Tests for parse_config()

    #[test]
    fn parse_config_non_existent_file_returns_defaults() {
        let path = Path::new("/non/existent/path/bitcoin.conf");
        let entries = parse_config(path).unwrap();

        assert!(!entries.is_empty());

        // All entries should have schema and be disabled
        for entry in &entries {
            assert!(entry.schema.is_some());
            assert!(!entry.enabled);
        }
    }

    #[test]
    fn parse_config_empty_file_returns_defaults() {
        let (_dir, path) = create_temp_config("");
        let entries = parse_config(&path).unwrap();

        assert!(!entries.is_empty());

        // All entries should be disabled (not set in config)
        let enabled_count = entries.iter().filter(|e| e.enabled).count();
        assert_eq!(enabled_count, 0);
    }

    #[test]
    fn parse_config_parses_bool_values() {
        let (_dir, path) = create_temp_config("txindex=1\nserver=0\n");
        let entries = parse_config(&path).unwrap();

        let txindex = entries.iter().find(|e| e.key == "txindex").unwrap();
        assert_eq!(txindex.value, "1");
        assert!(txindex.enabled);

        let server = entries.iter().find(|e| e.key == "server").unwrap();
        assert_eq!(server.value, "0");
        assert!(server.enabled);
    }

    #[test]
    fn parse_config_parses_int_values() {
        let (_dir, path) = create_temp_config("dbcache=1000\nport=8334\n");
        let entries = parse_config(&path).unwrap();

        let dbcache = entries.iter().find(|e| e.key == "dbcache").unwrap();
        assert_eq!(dbcache.value, "1000");
        assert!(dbcache.enabled);

        let port = entries.iter().find(|e| e.key == "port").unwrap();
        assert_eq!(port.value, "8334");
        assert!(port.enabled);
    }

    #[test]
    fn parse_config_parses_string_values() {
        let (_dir, path) = create_temp_config("rpcuser=myuser\nrpcpassword=mypassword\n");
        let entries = parse_config(&path).unwrap();

        let rpcuser = entries.iter().find(|e| e.key == "rpcuser").unwrap();
        assert_eq!(rpcuser.value, "myuser");
        assert!(rpcuser.enabled);

        let rpcpassword = entries.iter().find(|e| e.key == "rpcpassword").unwrap();
        assert_eq!(rpcpassword.value, "mypassword");
        assert!(rpcpassword.enabled);
    }

    #[test]
    fn parse_config_parses_path_values() {
        let (_dir, path) = create_temp_config("datadir=/home/user/.bitcoin\n");
        let entries = parse_config(&path).unwrap();

        let datadir = entries.iter().find(|e| e.key == "datadir").unwrap();
        assert_eq!(datadir.value, "/home/user/.bitcoin");
        assert!(datadir.enabled);
    }

    #[test]
    fn parse_config_parses_address_values() {
        let (_dir, path) = create_temp_config("zmqpubhashblock=tcp://127.0.0.1:28332\n");
        let entries = parse_config(&path).unwrap();

        let zmq = entries.iter().find(|e| e.key == "zmqpubhashblock").unwrap();
        assert_eq!(zmq.value, "tcp://127.0.0.1:28332");
        assert!(zmq.enabled);
    }

    #[test]
    fn parse_config_handles_unknown_keys() {
        // Use a section to ensure the config crate parses the key properly
        let (_dir, path) = create_temp_config("[main]\nunknownkey=unknownvalue\n");
        let entries = parse_config(&path).unwrap();

        let unknown = entries.iter().find(|e| e.key == "unknownkey");
        assert!(
            unknown.is_some(),
            "Unknown key should be present in entries"
        );

        let unknown = unknown.unwrap();
        assert_eq!(unknown.value, "unknownvalue");
        assert!(unknown.enabled);
        assert!(unknown.schema.is_none());
    }

    #[test]
    fn parse_config_handles_section_values() {
        let content = r#"
[main]
rpcport=8332

[test]
rpcport=18332
"#;
        let (_dir, path) = create_temp_config(content);
        let entries = parse_config(&path).unwrap();

        // Should find rpcport with first matching section value
        let rpcport = entries.iter().find(|e| e.key == "rpcport").unwrap();
        assert!(rpcport.enabled);
    }

    #[test]
    fn parse_config_preserves_schema_info() {
        let (_dir, path) = create_temp_config("txindex=1\n");
        let entries = parse_config(&path).unwrap();

        let txindex = entries.iter().find(|e| e.key == "txindex").unwrap();
        assert!(txindex.schema.is_some());

        let schema = txindex.schema.as_ref().unwrap();
        assert_eq!(schema.config_type, ConfigType::Bool);
        assert_eq!(schema.category, ConfigCategory::Core);
        assert!(!schema.description.is_empty());
    }

    #[test]
    fn parse_config_uses_defaults_for_unset_options() {
        let (_dir, path) = create_temp_config("txindex=1\n");
        let entries = parse_config(&path).unwrap();

        // dbcache should have default value since not set
        let dbcache = entries.iter().find(|e| e.key == "dbcache").unwrap();
        assert_eq!(dbcache.value, "450"); // default value
        assert!(!dbcache.enabled);
    }

    #[test]
    fn parse_config_handles_comments() {
        let content = r#"
# This is a comment
txindex=1
# Another comment
server=1
"#;
        let (_dir, path) = create_temp_config(content);
        let entries = parse_config(&path).unwrap();

        let txindex = entries.iter().find(|e| e.key == "txindex").unwrap();
        assert_eq!(txindex.value, "1");
        assert!(txindex.enabled);

        let server = entries.iter().find(|e| e.key == "server").unwrap();
        assert_eq!(server.value, "1");
        assert!(server.enabled);
    }

    #[test]
    fn parse_config_handles_full_config() {
        let content = r#"
# Bitcoin Core configuration

# Network
testnet=0
listen=1
port=8333
maxconnections=125

# RPC
server=1
rpcuser=bitcoinrpc
rpcpassword=secretpassword
rpcport=8332
rpcallowip=127.0.0.1

# Wallet
disablewallet=0
fallbackfee=0.0002

# ZMQ
zmqpubhashblock=tcp://127.0.0.1:28332
zmqpubhashtx=tcp://127.0.0.1:28333
"#;
        let (_dir, path) = create_temp_config(content);
        let entries = parse_config(&path).unwrap();

        // Verify various entries
        let testnet = entries.iter().find(|e| e.key == "testnet").unwrap();
        assert_eq!(testnet.value, "0");
        assert!(testnet.enabled);

        let rpcuser = entries.iter().find(|e| e.key == "rpcuser").unwrap();
        assert_eq!(rpcuser.value, "bitcoinrpc");

        let zmq = entries.iter().find(|e| e.key == "zmqpubhashblock").unwrap();
        assert_eq!(zmq.value, "tcp://127.0.0.1:28332");
    }

    // Tests for ConfigType and ConfigCategory enums

    #[test]
    fn config_type_is_copy() {
        let t1 = ConfigType::Bool;
        let t2 = t1; // Copy
        assert_eq!(t1, t2);
    }

    #[test]
    fn config_category_is_copy() {
        let c1 = ConfigCategory::Core;
        let c2 = c1; // Copy
        assert_eq!(c1, c2);
    }

    #[test]
    fn config_entry_clone_works() {
        let entry = ConfigEntry {
            key: "test".to_string(),
            value: "value".to_string(),
            schema: None,
            enabled: true,
        };
        let cloned = entry.clone();
        assert_eq!(entry.key, cloned.key);
        assert_eq!(entry.value, cloned.value);
        assert_eq!(entry.enabled, cloned.enabled);
    }

    #[test]
    fn config_schema_clone_works() {
        let schema = ConfigSchema::new(
            "test",
            "default",
            ConfigType::String,
            ConfigCategory::Core,
            "description",
        );
        let cloned = schema.clone();
        assert_eq!(schema.key, cloned.key);
        assert_eq!(schema.default, cloned.default);
        assert_eq!(schema.config_type, cloned.config_type);
        assert_eq!(schema.category, cloned.category);
        assert_eq!(schema.description, cloned.description);
    }
}
