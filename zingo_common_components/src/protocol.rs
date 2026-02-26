//! Module for types associated with the zcash protocol and consensus.

/// Network types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkType {
    /// Mainnet
    Mainnet,
    /// Testnet
    Testnet,
    /// Regtest
    Regtest(ActivationHeights),
}

impl std::fmt::Display for NetworkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chain = match self {
            NetworkType::Mainnet => "mainnet",
            NetworkType::Testnet => "testnet",
            NetworkType::Regtest(_) => "regtest",
        };
        write!(f, "{chain}")
    }
}

/// Network upgrade activation heights for custom testnet and regtest network configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivationHeights {
    /// Overwinter network upgrade activation height.
    pub overwinter: Option<u32>,

    /// Sapling network upgrade activation height.
    pub sapling: Option<u32>,

    /// Blossom network upgrade activation height.
    pub blossom: Option<u32>,

    /// Heartwood network upgrade activation height.
    pub heartwood: Option<u32>,

    /// Canopy network upgrade activation height.
    pub canopy: Option<u32>,

    /// Nu5 network upgrade activation height.
    pub nu5: Option<u32>,

    /// Nu6 network upgrade activation height.
    pub nu6: Option<u32>,

    /// Nu6.1 network upgrade activation height.
    pub nu6_1: Option<u32>,

    /// Nu7 network upgrade activation height.
    pub nu7: Option<u32>,
}

impl Default for ActivationHeights {
    fn default() -> Self {
        Self {
            overwinter: Some(1),
            sapling: Some(1),
            blossom: Some(1),
            heartwood: Some(1),
            canopy: Some(1),
            nu5: Some(1),
            nu6: Some(1),
            nu6_1: Some(1),
            nu7: None,
        }
    }
}

impl ActivationHeights {
    /// Returns overwinter network upgrade activation height.
    pub fn overwinter(&self) -> Option<u32> {
        self.overwinter
    }

    /// Returns sapling network upgrade activation height.
    pub fn sapling(&self) -> Option<u32> {
        self.sapling
    }

    /// Returns blossom network upgrade activation height.
    pub fn blossom(&self) -> Option<u32> {
        self.blossom
    }

    /// Returns heartwood network upgrade activation height.
    pub fn heartwood(&self) -> Option<u32> {
        self.heartwood
    }

    /// Returns canopy network upgrade activation height.
    pub fn canopy(&self) -> Option<u32> {
        self.canopy
    }

    /// Returns nu5 network upgrade activation height.
    pub fn nu5(&self) -> Option<u32> {
        self.nu5
    }

    /// Returns nu6 network upgrade activation height.
    pub fn nu6(&self) -> Option<u32> {
        self.nu6
    }

    /// Returns nu6.1 network upgrade activation height.
    pub fn nu6_1(&self) -> Option<u32> {
        self.nu6_1
    }

    /// Returns nu7 network upgrade activation height.
    pub fn nu7(&self) -> Option<u32> {
        self.nu7
    }
}
