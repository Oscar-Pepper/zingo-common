/// Network upgrade activation heights for custom testnet and regtest network configuration.
pub struct ActivationHeights {
    overwinter: Option<u32>,
    sapling: Option<u32>,
    blossom: Option<u32>,
    heartwood: Option<u32>,
    canopy: Option<u32>,
    nu5: Option<u32>,
    nu6: Option<u32>,
    nu6_1: Option<u32>,
    nu7: Option<u32>,
}

impl Default for ActivationHeights {
    fn default() -> Self {
        Self::new(
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            None,
        )
    }
}

impl ActivationHeights {
    /// Constructor with assertions to ensure all earlier network upgrades are active with an activation height equal
    /// to or lower than the later network upgrades.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        overwinter: Option<u32>,
        sapling: Option<u32>,
        blossom: Option<u32>,
        heartwood: Option<u32>,
        canopy: Option<u32>,
        nu5: Option<u32>,
        nu6: Option<u32>,
        nu6_1: Option<u32>,
        nu7: Option<u32>,
    ) -> Self {
        if let Some(b) = sapling {
            assert!(overwinter.is_some_and(|a| a <= b));
        }
        if let Some(b) = blossom {
            assert!(sapling.is_some_and(|a| a <= b));
        }
        if let Some(b) = heartwood {
            assert!(blossom.is_some_and(|a| a <= b));
        }
        if let Some(b) = canopy {
            assert!(heartwood.is_some_and(|a| a <= b));
        }
        if let Some(b) = nu5 {
            assert!(canopy.is_some_and(|a| a <= b));
        }
        if let Some(b) = nu6 {
            assert!(nu5.is_some_and(|a| a <= b));
        }
        if let Some(b) = nu6_1 {
            assert!(nu6.is_some_and(|a| a <= b));
        }
        if let Some(b) = nu7 {
            assert!(nu6_1.is_some_and(|a| a <= b));
        }

        Self {
            overwinter,
            sapling,
            blossom,
            heartwood,
            canopy,
            nu5,
            nu6,
            nu6_1,
            nu7,
        }
    }

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
