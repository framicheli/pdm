// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result};
use config::{Config, File, FileFormat};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// Core Config
#[derive(Debug, Clone, Default)]
pub struct Core {
    // Data directory and storage
    pub datadir: Option<String>,
    pub blocksdir: Option<String>,
    pub pid: Option<String>,
    pub debuglogfile: Option<String>,
    pub settings: Option<String>,
    pub includeconf: Option<String>,
    pub loadblock: Option<String>,

    // Indexing
    pub txindex: Option<bool>,
    pub blockfilterindex: Option<String>,
    pub coinstatsindex: Option<bool>,

    // Pruning
    pub prune: Option<u32>,

    // Memory and performance
    pub dbcache: Option<u32>,
    pub maxmempool: Option<u32>,
    pub maxorphantx: Option<u32>,
    pub mempoolexpiry: Option<u32>,
    pub par: Option<i32>,
    pub blockreconstructionextratxn: Option<u32>,

    // Behavior
    pub blocksonly: Option<bool>,
    pub persistmempool: Option<bool>,
    pub reindex: Option<bool>,
    pub reindex_chainstate: Option<bool>,
    pub sysperms: Option<bool>,

    // Daemon mode
    pub daemon: Option<bool>,
    pub daemonwait: Option<bool>,

    // Notification commands
    pub alertnotify: Option<String>,
    pub blocknotify: Option<String>,
    pub startupnotify: Option<String>,

    // Validation
    pub assumevalid: Option<String>,
}

/// Network Config
#[derive(Debug, Clone, Default)]
pub struct Network {
    // Chain selection
    pub chain: Option<String>,
    pub testnet: Option<bool>,
    pub regtest: Option<bool>,
    pub signet: Option<bool>,
    pub signetchallenge: Option<String>,
    pub signetseednode: Option<String>,

    // Listening and binding
    pub listen: Option<bool>,
    pub bind: Option<String>,
    pub whitebind: Option<String>,
    pub port: Option<u32>,

    // Connection limits
    pub maxconnections: Option<u32>,
    pub maxreceivebuffer: Option<u32>,
    pub maxsendbuffer: Option<u32>,
    pub maxuploadtarget: Option<u32>,
    pub timeout: Option<u32>,
    pub maxtimeadjustment: Option<u32>,
    pub bantime: Option<u32>,

    // Peer discovery
    pub discover: Option<bool>,
    pub dns: Option<bool>,
    pub dnsseed: Option<bool>,
    pub fixedseeds: Option<bool>,
    pub forcednsseed: Option<bool>,
    pub seednode: Option<String>,
    pub addnode: Option<String>,
    pub connect: Option<String>,

    // Network selection
    pub onlynet: Option<String>,
    pub networkactive: Option<bool>,

    // Proxy settings
    pub proxy: Option<String>,
    pub proxyrandomize: Option<bool>,

    // Tor settings
    pub onion: Option<String>,
    pub listenonion: Option<bool>,
    pub torcontrol: Option<String>,
    pub torpassword: Option<String>,

    // I2P settings
    pub i2psam: Option<String>,
    pub i2pacceptincoming: Option<bool>,

    // CJDNS
    pub cjdnsreachable: Option<bool>,

    // Peer permissions
    pub whitelist: Option<String>,
    pub peerblockfilters: Option<bool>,
    pub peerbloomfilters: Option<bool>,
    pub permitbaremultisig: Option<bool>,

    // External IP
    pub externalip: Option<String>,

    // UPnP
    pub upnp: Option<bool>,

    // ASN mapping
    pub asmap: Option<String>,
}

/// RPC Config
#[derive(Debug, Clone, Default)]
pub struct Rpc {
    // Server enable
    pub server: Option<bool>,

    // Authentication
    pub rpcuser: Option<String>,
    pub rpcpassword: Option<String>,
    pub rpcauth: Option<String>,
    pub rpccookiefile: Option<String>,

    // Connection
    pub rpcport: Option<u32>,
    pub rpcbind: Option<String>,
    pub rpcallowip: Option<String>,

    // Performance
    pub rpcthreads: Option<u32>,

    // Serialization
    pub rpcserialversion: Option<u32>,

    // Whitelist
    pub rpcwhitelist: Option<String>,
    pub rpcwhitelistdefault: Option<bool>,

    // REST interface
    pub rest: Option<bool>,
}

/// Wallet related config
#[derive(Debug, Clone, Default)]
pub struct Wallet {
    // Enable/disable
    pub disablewallet: Option<bool>,

    // Wallet paths
    pub wallet: Option<String>,
    pub walletdir: Option<String>,

    // Address types
    pub addresstype: Option<String>,
    pub changetype: Option<String>,

    // Fee settings
    pub fallbackfee: Option<String>,
    pub discardfee: Option<String>,
    pub mintxfee: Option<String>,
    pub paytxfee: Option<String>,
    pub consolidatefeerate: Option<String>,
    pub maxapsfee: Option<String>,

    // Transaction behavior
    pub txconfirmtarget: Option<u32>,
    pub spendzeroconfchange: Option<bool>,
    pub walletrbf: Option<bool>,
    pub avoidpartialspends: Option<bool>,

    // Key management
    pub keypool: Option<u32>,

    // External signer
    pub signer: Option<String>,

    // Broadcast
    pub walletbroadcast: Option<bool>,

    // Notifications
    pub walletnotify: Option<String>,
}

/// Debugging related config
#[derive(Debug, Clone, Default)]
pub struct Debugging {
    // Debug categories
    pub debug: Option<String>,
    pub debugexclude: Option<String>,

    // Logging options
    pub logips: Option<bool>,
    pub logsourcelocations: Option<bool>,
    pub logthreadnames: Option<bool>,
    pub logtimestamps: Option<bool>,
    pub shrinkdebugfile: Option<bool>,
    pub printtoconsole: Option<bool>,

    // User agent
    pub uacomment: Option<String>,

    // Fee limits
    pub maxtxfee: Option<String>,
}

/// Mining related config
#[derive(Debug, Clone, Default)]
pub struct Mining {
    // Block creation
    pub blockmaxweight: Option<u32>,
    pub blockmintxfee: Option<String>,
}

/// Relay related config
#[derive(Debug, Clone, Default)]
pub struct Relay {
    // Relay fees
    pub minrelaytxfee: Option<String>,

    // Data carrier (OP_RETURN)
    pub datacarrier: Option<bool>,
    pub datacarriersize: Option<u32>,

    // Sigops
    pub bytespersigop: Option<u32>,

    // Whitelist relay
    pub whitelistforcerelay: Option<bool>,
    pub whitelistrelay: Option<bool>,
}

/// ZMQ related config
#[derive(Debug, Clone, Default)]
pub struct Zmq {
    // Hash notifications
    pub zmqpubhashblock: Option<String>,
    pub zmqpubhashtx: Option<String>,

    // Raw data notifications
    pub zmqpubrawblock: Option<String>,
    pub zmqpubrawtx: Option<String>,

    // Sequence notifications
    pub zmqpubsequence: Option<String>,
}

/// Complete Bitcoin configuration
#[derive(Debug, Clone, Default)]
pub struct BitcoinConfig {
    pub path: Option<PathBuf>,
    pub core: Core,
    pub network: Network,
    pub rpc: Rpc,
    pub wallet: Wallet,
    pub debugging: Debugging,
    pub mining: Mining,
    pub relay: Relay,
    pub zmq: Zmq,
}

impl BitcoinConfig {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Open and parse bitcoin.conf file
    pub fn open(path: &Path) -> Result<Self> {
        let mut config = Self {
            path: Some(path.to_path_buf()),
            ..Default::default()
        };

        // If path doesn't exists error
        // TODO: App specific error?
        if !path.exists() {
            eprintln!("Path doesn't exist!");
        }

        let builder = Config::builder().add_source(File::from(path).format(FileFormat::Ini));

        let parsed = match builder.build() {
            Ok(cfg) => cfg,
            Err(_) => return Ok(config),
        };

        let sections = vec!["", "main", "test", "signet", "regtest"];

        macro_rules! get_string {
            ($key:expr) => {{
                let mut result = None;
                for section in &sections {
                    let lookup = if section.is_empty() {
                        $key.to_string()
                    } else {
                        format!("{}.{}", section, $key)
                    };
                    if let Ok(val) = parsed.get_string(&lookup) {
                        result = Some(val);
                        break;
                    }
                }
                result
            }};
        }

        macro_rules! get_bool {
            ($key:expr) => {{
                let mut result = None;
                for section in &sections {
                    let lookup = if section.is_empty() {
                        $key.to_string()
                    } else {
                        format!("{}.{}", section, $key)
                    };
                    if let Ok(val) = parsed.get_bool(&lookup) {
                        result = Some(val);
                        break;
                    }
                    if let Ok(val) = parsed.get_int(&lookup) {
                        result = Some(val != 0);
                        break;
                    }
                }
                result
            }};
        }

        macro_rules! get_u32 {
            ($key:expr) => {{
                let mut result = None;
                for section in &sections {
                    let lookup = if section.is_empty() {
                        $key.to_string()
                    } else {
                        format!("{}.{}", section, $key)
                    };
                    if let Ok(val) = parsed.get_int(&lookup) {
                        result = Some(val as u32);
                        break;
                    }
                }
                result
            }};
        }

        macro_rules! get_i32 {
            ($key:expr) => {{
                let mut result = None;
                for section in &sections {
                    let lookup = if section.is_empty() {
                        $key.to_string()
                    } else {
                        format!("{}.{}", section, $key)
                    };
                    if let Ok(val) = parsed.get_int(&lookup) {
                        result = Some(val as i32);
                        break;
                    }
                }
                result
            }};
        }

        // Core options
        config.core.datadir = get_string!("datadir");
        config.core.blocksdir = get_string!("blocksdir");
        config.core.pid = get_string!("pid");
        config.core.debuglogfile = get_string!("debuglogfile");
        config.core.settings = get_string!("settings");
        config.core.includeconf = get_string!("includeconf");
        config.core.loadblock = get_string!("loadblock");
        config.core.txindex = get_bool!("txindex");
        config.core.blockfilterindex = get_string!("blockfilterindex");
        config.core.coinstatsindex = get_bool!("coinstatsindex");
        config.core.prune = get_u32!("prune");
        config.core.dbcache = get_u32!("dbcache");
        config.core.maxmempool = get_u32!("maxmempool");
        config.core.maxorphantx = get_u32!("maxorphantx");
        config.core.mempoolexpiry = get_u32!("mempoolexpiry");
        config.core.par = get_i32!("par");
        config.core.blockreconstructionextratxn = get_u32!("blockreconstructionextratxn");
        config.core.blocksonly = get_bool!("blocksonly");
        config.core.persistmempool = get_bool!("persistmempool");
        config.core.reindex = get_bool!("reindex");
        config.core.reindex_chainstate = get_bool!("reindex-chainstate");
        config.core.sysperms = get_bool!("sysperms");
        config.core.daemon = get_bool!("daemon");
        config.core.daemonwait = get_bool!("daemonwait");
        config.core.alertnotify = get_string!("alertnotify");
        config.core.blocknotify = get_string!("blocknotify");
        config.core.startupnotify = get_string!("startupnotify");
        config.core.assumevalid = get_string!("assumevalid");

        // Network options
        config.network.chain = get_string!("chain");
        config.network.testnet = get_bool!("testnet");
        config.network.regtest = get_bool!("regtest");
        config.network.signet = get_bool!("signet");
        config.network.signetchallenge = get_string!("signetchallenge");
        config.network.signetseednode = get_string!("signetseednode");
        config.network.listen = get_bool!("listen");
        config.network.bind = get_string!("bind");
        config.network.whitebind = get_string!("whitebind");
        config.network.port = get_u32!("port");
        config.network.maxconnections = get_u32!("maxconnections");
        config.network.maxreceivebuffer = get_u32!("maxreceivebuffer");
        config.network.maxsendbuffer = get_u32!("maxsendbuffer");
        config.network.maxuploadtarget = get_u32!("maxuploadtarget");
        config.network.timeout = get_u32!("timeout");
        config.network.maxtimeadjustment = get_u32!("maxtimeadjustment");
        config.network.bantime = get_u32!("bantime");
        config.network.discover = get_bool!("discover");
        config.network.dns = get_bool!("dns");
        config.network.dnsseed = get_bool!("dnsseed");
        config.network.fixedseeds = get_bool!("fixedseeds");
        config.network.forcednsseed = get_bool!("forcednsseed");
        config.network.seednode = get_string!("seednode");
        config.network.addnode = get_string!("addnode");
        config.network.connect = get_string!("connect");
        config.network.onlynet = get_string!("onlynet");
        config.network.networkactive = get_bool!("networkactive");
        config.network.proxy = get_string!("proxy");
        config.network.proxyrandomize = get_bool!("proxyrandomize");
        config.network.onion = get_string!("onion");
        config.network.listenonion = get_bool!("listenonion");
        config.network.torcontrol = get_string!("torcontrol");
        config.network.torpassword = get_string!("torpassword");
        config.network.i2psam = get_string!("i2psam");
        config.network.i2pacceptincoming = get_bool!("i2pacceptincoming");
        config.network.cjdnsreachable = get_bool!("cjdnsreachable");
        config.network.whitelist = get_string!("whitelist");
        config.network.peerblockfilters = get_bool!("peerblockfilters");
        config.network.peerbloomfilters = get_bool!("peerbloomfilters");
        config.network.permitbaremultisig = get_bool!("permitbaremultisig");
        config.network.externalip = get_string!("externalip");
        config.network.upnp = get_bool!("upnp");
        config.network.asmap = get_string!("asmap");

        // RPC options
        config.rpc.server = get_bool!("server");
        config.rpc.rpcuser = get_string!("rpcuser");
        config.rpc.rpcpassword = get_string!("rpcpassword");
        config.rpc.rpcauth = get_string!("rpcauth");
        config.rpc.rpccookiefile = get_string!("rpccookiefile");
        config.rpc.rpcport = get_u32!("rpcport");
        config.rpc.rpcbind = get_string!("rpcbind");
        config.rpc.rpcallowip = get_string!("rpcallowip");
        config.rpc.rpcthreads = get_u32!("rpcthreads");
        config.rpc.rpcserialversion = get_u32!("rpcserialversion");
        config.rpc.rpcwhitelist = get_string!("rpcwhitelist");
        config.rpc.rpcwhitelistdefault = get_bool!("rpcwhitelistdefault");
        config.rpc.rest = get_bool!("rest");

        // Wallet options
        config.wallet.disablewallet = get_bool!("disablewallet");
        config.wallet.wallet = get_string!("wallet");
        config.wallet.walletdir = get_string!("walletdir");
        config.wallet.addresstype = get_string!("addresstype");
        config.wallet.changetype = get_string!("changetype");
        config.wallet.fallbackfee = get_string!("fallbackfee");
        config.wallet.discardfee = get_string!("discardfee");
        config.wallet.mintxfee = get_string!("mintxfee");
        config.wallet.paytxfee = get_string!("paytxfee");
        config.wallet.consolidatefeerate = get_string!("consolidatefeerate");
        config.wallet.maxapsfee = get_string!("maxapsfee");
        config.wallet.txconfirmtarget = get_u32!("txconfirmtarget");
        config.wallet.spendzeroconfchange = get_bool!("spendzeroconfchange");
        config.wallet.walletrbf = get_bool!("walletrbf");
        config.wallet.avoidpartialspends = get_bool!("avoidpartialspends");
        config.wallet.keypool = get_u32!("keypool");
        config.wallet.signer = get_string!("signer");
        config.wallet.walletbroadcast = get_bool!("walletbroadcast");
        config.wallet.walletnotify = get_string!("walletnotify");

        // Debugging options
        config.debugging.debug = get_string!("debug");
        config.debugging.debugexclude = get_string!("debugexclude");
        config.debugging.logips = get_bool!("logips");
        config.debugging.logsourcelocations = get_bool!("logsourcelocations");
        config.debugging.logthreadnames = get_bool!("logthreadnames");
        config.debugging.logtimestamps = get_bool!("logtimestamps");
        config.debugging.shrinkdebugfile = get_bool!("shrinkdebugfile");
        config.debugging.printtoconsole = get_bool!("printtoconsole");
        config.debugging.uacomment = get_string!("uacomment");
        config.debugging.maxtxfee = get_string!("maxtxfee");

        // Mining options
        config.mining.blockmaxweight = get_u32!("blockmaxweight");
        config.mining.blockmintxfee = get_string!("blockmintxfee");

        // Relay options
        config.relay.minrelaytxfee = get_string!("minrelaytxfee");
        config.relay.datacarrier = get_bool!("datacarrier");
        config.relay.datacarriersize = get_u32!("datacarriersize");
        config.relay.bytespersigop = get_u32!("bytespersigop");
        config.relay.whitelistforcerelay = get_bool!("whitelistforcerelay");
        config.relay.whitelistrelay = get_bool!("whitelistrelay");

        // ZMQ options
        config.zmq.zmqpubhashblock = get_string!("zmqpubhashblock");
        config.zmq.zmqpubhashtx = get_string!("zmqpubhashtx");
        config.zmq.zmqpubrawblock = get_string!("zmqpubrawblock");
        config.zmq.zmqpubrawtx = get_string!("zmqpubrawtx");
        config.zmq.zmqpubsequence = get_string!("zmqpubsequence");

        Ok(config)
    }

    /// Save the configuration to the stored path
    pub fn save(&self) -> Result<()> {
        let path = self
            .path
            .as_ref()
            .context("No path set for configuration")?;
        self.save_to(path)
    }

    /// Save the configuration to a specific path
    pub fn save_to(&self, path: &Path) -> Result<()> {
        let content = self.to_config_string();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let mut file = fs::File::create(path)
            .with_context(|| format!("Failed to create config file: {:?}", path))?;

        file.write_all(content.as_bytes())
            .with_context(|| format!("Failed to write to config file: {:?}", path))?;

        Ok(())
    }

    /// Convert the configuration to a bitcoin.conf formatted string
    pub fn to_config_string(&self) -> String {
        let mut output = String::new();

        // Helper macro to write options
        macro_rules! write_string {
            ($output:expr, $key:expr, $value:expr) => {
                if let Some(ref val) = $value {
                    $output.push_str(&format!("{}={}\n", $key, val));
                }
            };
        }

        macro_rules! write_bool {
            ($output:expr, $key:expr, $value:expr) => {
                if let Some(val) = $value {
                    $output.push_str(&format!("{}={}\n", $key, if val { "1" } else { "0" }));
                }
            };
        }

        macro_rules! write_u32 {
            ($output:expr, $key:expr, $value:expr) => {
                if let Some(val) = $value {
                    $output.push_str(&format!("{}={}\n", $key, val));
                }
            };
        }

        macro_rules! write_i32 {
            ($output:expr, $key:expr, $value:expr) => {
                if let Some(val) = $value {
                    $output.push_str(&format!("{}={}\n", $key, val));
                }
            };
        }

        // Core options
        output.push_str("# Core\n");
        write_string!(output, "datadir", self.core.datadir);
        write_string!(output, "blocksdir", self.core.blocksdir);
        write_string!(output, "pid", self.core.pid);
        write_string!(output, "debuglogfile", self.core.debuglogfile);
        write_string!(output, "settings", self.core.settings);
        write_string!(output, "includeconf", self.core.includeconf);
        write_string!(output, "loadblock", self.core.loadblock);
        write_bool!(output, "txindex", self.core.txindex);
        write_string!(output, "blockfilterindex", self.core.blockfilterindex);
        write_bool!(output, "coinstatsindex", self.core.coinstatsindex);
        write_u32!(output, "prune", self.core.prune);
        write_u32!(output, "dbcache", self.core.dbcache);
        write_u32!(output, "maxmempool", self.core.maxmempool);
        write_u32!(output, "maxorphantx", self.core.maxorphantx);
        write_u32!(output, "mempoolexpiry", self.core.mempoolexpiry);
        write_i32!(output, "par", self.core.par);
        write_u32!(
            output,
            "blockreconstructionextratxn",
            self.core.blockreconstructionextratxn
        );
        write_bool!(output, "blocksonly", self.core.blocksonly);
        write_bool!(output, "persistmempool", self.core.persistmempool);
        write_bool!(output, "reindex", self.core.reindex);
        write_bool!(output, "reindex-chainstate", self.core.reindex_chainstate);
        write_bool!(output, "sysperms", self.core.sysperms);
        write_bool!(output, "daemon", self.core.daemon);
        write_bool!(output, "daemonwait", self.core.daemonwait);
        write_string!(output, "alertnotify", self.core.alertnotify);
        write_string!(output, "blocknotify", self.core.blocknotify);
        write_string!(output, "startupnotify", self.core.startupnotify);
        write_string!(output, "assumevalid", self.core.assumevalid);

        // Network options
        output.push_str("\n# Network\n");
        write_string!(output, "chain", self.network.chain);
        write_bool!(output, "testnet", self.network.testnet);
        write_bool!(output, "regtest", self.network.regtest);
        write_bool!(output, "signet", self.network.signet);
        write_string!(output, "signetchallenge", self.network.signetchallenge);
        write_string!(output, "signetseednode", self.network.signetseednode);
        write_bool!(output, "listen", self.network.listen);
        write_string!(output, "bind", self.network.bind);
        write_string!(output, "whitebind", self.network.whitebind);
        write_u32!(output, "port", self.network.port);
        write_u32!(output, "maxconnections", self.network.maxconnections);
        write_u32!(output, "maxreceivebuffer", self.network.maxreceivebuffer);
        write_u32!(output, "maxsendbuffer", self.network.maxsendbuffer);
        write_u32!(output, "maxuploadtarget", self.network.maxuploadtarget);
        write_u32!(output, "timeout", self.network.timeout);
        write_u32!(output, "maxtimeadjustment", self.network.maxtimeadjustment);
        write_u32!(output, "bantime", self.network.bantime);
        write_bool!(output, "discover", self.network.discover);
        write_bool!(output, "dns", self.network.dns);
        write_bool!(output, "dnsseed", self.network.dnsseed);
        write_bool!(output, "fixedseeds", self.network.fixedseeds);
        write_bool!(output, "forcednsseed", self.network.forcednsseed);
        write_string!(output, "seednode", self.network.seednode);
        write_string!(output, "addnode", self.network.addnode);
        write_string!(output, "connect", self.network.connect);
        write_string!(output, "onlynet", self.network.onlynet);
        write_bool!(output, "networkactive", self.network.networkactive);
        write_string!(output, "proxy", self.network.proxy);
        write_bool!(output, "proxyrandomize", self.network.proxyrandomize);
        write_string!(output, "onion", self.network.onion);
        write_bool!(output, "listenonion", self.network.listenonion);
        write_string!(output, "torcontrol", self.network.torcontrol);
        write_string!(output, "torpassword", self.network.torpassword);
        write_string!(output, "i2psam", self.network.i2psam);
        write_bool!(output, "i2pacceptincoming", self.network.i2pacceptincoming);
        write_bool!(output, "cjdnsreachable", self.network.cjdnsreachable);
        write_string!(output, "whitelist", self.network.whitelist);
        write_bool!(output, "peerblockfilters", self.network.peerblockfilters);
        write_bool!(output, "peerbloomfilters", self.network.peerbloomfilters);
        write_bool!(
            output,
            "permitbaremultisig",
            self.network.permitbaremultisig
        );
        write_string!(output, "externalip", self.network.externalip);
        write_bool!(output, "upnp", self.network.upnp);
        write_string!(output, "asmap", self.network.asmap);

        // RPC options
        output.push_str("\n# RPC\n");
        write_bool!(output, "server", self.rpc.server);
        write_string!(output, "rpcuser", self.rpc.rpcuser);
        write_string!(output, "rpcpassword", self.rpc.rpcpassword);
        write_string!(output, "rpcauth", self.rpc.rpcauth);
        write_string!(output, "rpccookiefile", self.rpc.rpccookiefile);
        write_u32!(output, "rpcport", self.rpc.rpcport);
        write_string!(output, "rpcbind", self.rpc.rpcbind);
        write_string!(output, "rpcallowip", self.rpc.rpcallowip);
        write_u32!(output, "rpcthreads", self.rpc.rpcthreads);
        write_u32!(output, "rpcserialversion", self.rpc.rpcserialversion);
        write_string!(output, "rpcwhitelist", self.rpc.rpcwhitelist);
        write_bool!(output, "rpcwhitelistdefault", self.rpc.rpcwhitelistdefault);
        write_bool!(output, "rest", self.rpc.rest);

        // Wallet options
        output.push_str("\n# Wallet\n");
        write_bool!(output, "disablewallet", self.wallet.disablewallet);
        write_string!(output, "wallet", self.wallet.wallet);
        write_string!(output, "walletdir", self.wallet.walletdir);
        write_string!(output, "addresstype", self.wallet.addresstype);
        write_string!(output, "changetype", self.wallet.changetype);
        write_string!(output, "fallbackfee", self.wallet.fallbackfee);
        write_string!(output, "discardfee", self.wallet.discardfee);
        write_string!(output, "mintxfee", self.wallet.mintxfee);
        write_string!(output, "paytxfee", self.wallet.paytxfee);
        write_string!(output, "consolidatefeerate", self.wallet.consolidatefeerate);
        write_string!(output, "maxapsfee", self.wallet.maxapsfee);
        write_u32!(output, "txconfirmtarget", self.wallet.txconfirmtarget);
        write_bool!(
            output,
            "spendzeroconfchange",
            self.wallet.spendzeroconfchange
        );
        write_bool!(output, "walletrbf", self.wallet.walletrbf);
        write_bool!(output, "avoidpartialspends", self.wallet.avoidpartialspends);
        write_u32!(output, "keypool", self.wallet.keypool);
        write_string!(output, "signer", self.wallet.signer);
        write_bool!(output, "walletbroadcast", self.wallet.walletbroadcast);
        write_string!(output, "walletnotify", self.wallet.walletnotify);

        // Debugging options
        output.push_str("\n# Debugging\n");
        write_string!(output, "debug", self.debugging.debug);
        write_string!(output, "debugexclude", self.debugging.debugexclude);
        write_bool!(output, "logips", self.debugging.logips);
        write_bool!(
            output,
            "logsourcelocations",
            self.debugging.logsourcelocations
        );
        write_bool!(output, "logthreadnames", self.debugging.logthreadnames);
        write_bool!(output, "logtimestamps", self.debugging.logtimestamps);
        write_bool!(output, "shrinkdebugfile", self.debugging.shrinkdebugfile);
        write_bool!(output, "printtoconsole", self.debugging.printtoconsole);
        write_string!(output, "uacomment", self.debugging.uacomment);
        write_string!(output, "maxtxfee", self.debugging.maxtxfee);

        // Mining options
        output.push_str("\n# Mining\n");
        write_u32!(output, "blockmaxweight", self.mining.blockmaxweight);
        write_string!(output, "blockmintxfee", self.mining.blockmintxfee);

        // Relay options
        output.push_str("\n# Relay\n");
        write_string!(output, "minrelaytxfee", self.relay.minrelaytxfee);
        write_bool!(output, "datacarrier", self.relay.datacarrier);
        write_u32!(output, "datacarriersize", self.relay.datacarriersize);
        write_u32!(output, "bytespersigop", self.relay.bytespersigop);
        write_bool!(
            output,
            "whitelistforcerelay",
            self.relay.whitelistforcerelay
        );
        write_bool!(output, "whitelistrelay", self.relay.whitelistrelay);

        // ZMQ options
        output.push_str("\n# ZMQ\n");
        write_string!(output, "zmqpubhashblock", self.zmq.zmqpubhashblock);
        write_string!(output, "zmqpubhashtx", self.zmq.zmqpubhashtx);
        write_string!(output, "zmqpubrawblock", self.zmq.zmqpubrawblock);
        write_string!(output, "zmqpubrawtx", self.zmq.zmqpubrawtx);
        write_string!(output, "zmqpubsequence", self.zmq.zmqpubsequence);

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_temp_config(content: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("bitcoin.conf");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        (dir, file_path)
    }

    #[test]
    fn new_creates_empty_config() {
        let config = BitcoinConfig::new();
        assert!(config.path.is_none());
        assert!(config.core.txindex.is_none());
        assert!(config.rpc.server.is_none());
    }

    #[test]
    fn open_non_existent_returns_empty() {
        let path = Path::new("/non/existent/path/bitcoin.conf");
        let config = BitcoinConfig::open(path).unwrap();

        assert!(config.core.txindex.is_none());
        assert!(config.rpc.server.is_none());
    }

    #[test]
    fn open_empty_file_returns_empty() {
        let (_dir, path) = create_temp_config("");
        let config = BitcoinConfig::open(&path).unwrap();

        assert!(config.core.txindex.is_none());
        assert!(config.rpc.server.is_none());
    }

    #[test]
    fn open_parses_bool_values() {
        let (_dir, path) = create_temp_config("txindex=1\nserver=0\n");
        let config = BitcoinConfig::open(&path).unwrap();

        assert_eq!(config.core.txindex, Some(true));
        assert_eq!(config.rpc.server, Some(false));
    }

    #[test]
    fn open_parses_int_values() {
        let (_dir, path) = create_temp_config("dbcache=1000\nport=8334\n");
        let config = BitcoinConfig::open(&path).unwrap();

        assert_eq!(config.core.dbcache, Some(1000));
        assert_eq!(config.network.port, Some(8334));
    }

    #[test]
    fn open_parses_string_values() {
        let (_dir, path) = create_temp_config("rpcuser=myuser\nrpcpassword=mypassword\n");
        let config = BitcoinConfig::open(&path).unwrap();

        assert_eq!(config.rpc.rpcuser, Some("myuser".to_string()));
        assert_eq!(config.rpc.rpcpassword, Some("mypassword".to_string()));
    }

    #[test]
    fn open_parses_path_values() {
        let (_dir, path) = create_temp_config("datadir=/home/user/.bitcoin\n");
        let config = BitcoinConfig::open(&path).unwrap();

        assert_eq!(config.core.datadir, Some("/home/user/.bitcoin".to_string()));
    }

    #[test]
    fn open_parses_address_values() {
        let (_dir, path) = create_temp_config("zmqpubhashblock=tcp://127.0.0.1:28332\n");
        let config = BitcoinConfig::open(&path).unwrap();

        assert_eq!(
            config.zmq.zmqpubhashblock,
            Some("tcp://127.0.0.1:28332".to_string())
        );
    }

    #[test]
    fn open_handles_section_values() {
        let content = r#"
[main]
rpcport=8332
server=1
"#;
        let (_dir, path) = create_temp_config(content);
        let config = BitcoinConfig::open(&path).unwrap();

        assert_eq!(config.rpc.rpcport, Some(8332));
        assert_eq!(config.rpc.server, Some(true));
    }

    #[test]
    fn open_handles_comments() {
        let content = r#"
# This is a comment
txindex=1
# Another comment
server=1
"#;
        let (_dir, path) = create_temp_config(content);
        let config = BitcoinConfig::open(&path).unwrap();

        assert_eq!(config.core.txindex, Some(true));
        assert_eq!(config.rpc.server, Some(true));
    }

    #[test]
    fn save_and_reload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bitcoin.conf");

        let mut config = BitcoinConfig::new();
        config.path = Some(path.clone());
        config.core.txindex = Some(true);
        config.core.dbcache = Some(1000);
        config.rpc.server = Some(true);
        config.rpc.rpcport = Some(8332);
        config.save().unwrap();

        let reloaded = BitcoinConfig::open(&path).unwrap();

        assert_eq!(reloaded.core.txindex, Some(true));
        assert_eq!(reloaded.core.dbcache, Some(1000));
        assert_eq!(reloaded.rpc.server, Some(true));
        assert_eq!(reloaded.rpc.rpcport, Some(8332));
    }

    #[test]
    fn save_to_different_path() {
        let dir = tempfile::tempdir().unwrap();
        let new_path = dir.path().join("subdir/bitcoin_backup.conf");

        let mut config = BitcoinConfig::new();
        config.core.txindex = Some(true);

        config.save_to(&new_path).unwrap();

        assert!(new_path.exists());

        let reloaded = BitcoinConfig::open(&new_path).unwrap();
        assert_eq!(reloaded.core.txindex, Some(true));
    }

    #[test]
    fn to_config_string_outputs_set_values() {
        let mut config = BitcoinConfig::new();
        config.core.txindex = Some(true);
        config.rpc.server = Some(true);
        config.core.dbcache = Some(500);

        let output = config.to_config_string();

        assert!(output.contains("txindex=1"));
        assert!(output.contains("server=1"));
        assert!(output.contains("dbcache=500"));
        assert!(output.contains("# Core"));
        assert!(output.contains("# RPC"));
    }

    #[test]
    fn to_config_string_skips_none_values() {
        let config = BitcoinConfig::new();
        let output = config.to_config_string();

        assert!(!output.contains("txindex="));
        assert!(!output.contains("server="));
    }

    #[test]
    fn clone_works() {
        let mut config = BitcoinConfig::new();
        config.core.txindex = Some(true);
        config.rpc.rpcuser = Some("user".to_string());

        let cloned = config.clone();

        assert_eq!(cloned.core.txindex, Some(true));
        assert_eq!(cloned.rpc.rpcuser, Some("user".to_string()));
    }

    #[test]
    fn default_creates_empty_config() {
        let config = BitcoinConfig::default();

        assert!(config.path.is_none());
        assert!(config.core.txindex.is_none());
        assert!(config.network.listen.is_none());
        assert!(config.rpc.server.is_none());
    }
}
