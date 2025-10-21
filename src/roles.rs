// Add Quest role that only receives messages
#[derive(Debug, Clone)]
pub enum Roles {
    Admin,
    User,
}

#[derive(Debug, Clone)]
pub struct Permission {
    pub send: bool,
    pub receive: bool,
    pub kick: bool,
    pub ban: bool,
    pub unban: bool,
    pub mute: bool,
}

impl Roles {
    pub fn permissions(&self) -> Permission {
        match self {
            Roles::Admin => Permission {
                send: true,
                receive: true,
                kick: true,
                ban: true,
                unban: true,
                mute: true,
            },
            Roles::User => Permission {
                send: true,
                receive: true,
                kick: false,
                ban: false,
                unban: false,
                mute: false,
            },
        }
    }
}
