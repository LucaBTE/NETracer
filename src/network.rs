use netdev::get_default_interface;

pub struct NetworkInfo {
    pub hostname: String,
    pub interface: Option<String>,
    pub ipv4: Option<String>,
    pub gateway: Option<String>,
}

impl NetworkInfo {
    pub fn discover() -> Self {
        //hostname might fail, so I use "unknown" as fallback
        let hostname = hostname::get()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_string());

        //look for interface used by default route
        let Ok(interface) = get_default_interface() else {
            return Self {
                hostname,
                interface: None,
                ipv4: None,
                gateway: None,
            };
        };

        //takes the first IPv4 from the interface
        let ipv4 = interface
            .ipv4
            .first()
            .map(|network| network.addr().to_string());

        let gateway = interface
            .gateway
            .as_ref()
            .and_then(|gateway| gateway.ipv4.first())
            .map(|address| address.to_string());

        Self {
            hostname,
            interface: Some(interface.name),
            ipv4,
            gateway,
        }
    }

    pub fn has_link(&self) -> bool {
        self.interface.is_some() && self.ipv4.is_some()
    }
}
