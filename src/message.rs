pub enum Message {
    // info
    // info: <msg>
    Info {
        message: String,
    },
    // account
    // account: <id> <name> <protocol> <user> <status>
    Account {
        id: String,
        name: String,
        protocol: String,
        user: String,
        status: String,
    },
    // list accounts
    // account list
    AccountList,
    // add account
    // account add <protocol> <user> <password>
    AccountAdd {
        protocol: String,
        user: String,
        password: String,
    },
    // delete account
    // account <id> delete
    AccountDelete {
        id: String,
    },
    // buddy
    // buddy: <acc_id> status: <status> name: <name> alias: [alias]
    Buddy {
        account_id: String,
        status: String,
        name: String,
        alias: String,
    },
    // list buddies
    // account <id> buddies [online]
    BuddyList {
        account_id: String,
        status: String,
    },
    // message
    // message: <acc_id> <destination> <timestamp> <sender> <msg>
    Message {
        account_id: String,
        destination: String,
        timestamp: String,
        sender: String,
        message: String,
    },
    // collect (old) messages
    // account <id> collect
    MessageCollect {
        account_id: String,
    },
    // send message
    // account <id> send <user> <msg>
    MessageSend {
        account_id: String,
        destination: String,
        message: String,
    },
    // status
    // status: account <acc_id> status: <status>
    Status {
        account_id: String,
        status: String,
    },
    // get current status
    // account <id> status get
    StatusGet {
        account_id: String,
    },
    // set status
    // account <id> status set <status>
    StatusSet {
        account_id: String,
        status: String,
    },
    // chat
    // chat: list: <acc_id> <chat_id> <chat_alias> <nick>
    Chat {
        account_id: String,
        chat: String,
        alias: String,
        nick: String,
    },
    // list chats
    // account 0 chat list
    ChatList {
        account_id: String,
    },
    // join chat
    // account <id> chat join <chat>
    ChatJoin {
        account_id: String,
        chat: String,
    },
    // leave chat
    // account <id> chat part <chat>
    ChatLeave {
        account_id: String,
        chat: String,
    },
    // chat message
    // chat: msg: <acc_id> <chat> <timestamp> <sender> <message>
    ChatMessage {
        account_id: String,
        chat: String,
        timestamp: String,
        sender: String,
        message: String,
    },
    // send chat message
    // account <id> chat send <chat> <msg>
    ChatMessageSend {
        account_id: String,
        chat: String,
        message: String,
    },
    // chat user
    // chat: user: <acc_id> <chat> <name> <alias> <state>
    ChatUser {
        account_id: String,
        chat: String,
        user: String,
        alias: String,
        status: String,
    },
    // list chat users
    // account <id> chat users <chat>
    ChatUserList {
        account_id: String,
        chat: String,
    },
    // invite user to chat
    // account <id> chat invite <chat> <user>
    ChatUserInvite {
        account_id: String,
        chat: String,
        user: String,
    },
    // get version
    // version
    Version,
    // disconnect
    // bye
    Bye,
    // shutdown
    // quit
    Quit,
    // get help
    // help
    Help,
}

//impl std::str::FromStr for Message {
//    type Err = ();
//
//    fn from_str(s: &str) -> Result<Self, Self::Err> {
//        Ok(Message::Info { message: s.into() })
//    }
//}

fn parse(s: &str) -> Option<Message> {
    // messages:
    //
    let s = &s[..s.len() - 2];
    let s: Vec<&str> = s.split(' ').collect();
    if s.len() == 0 {
        return None;
    }
    match s[0] {
        // TODO: error?
        // info: <msg>
        "info:" => parse_info(s),
        // account: <id> <name> <protocol> <user> <status>
        "account:" => parse_account(s),
        // account list
        // account add <protocol> <user> <password>
        // account <id> delete
        // account <id> buddies [online]
        // account <id> collect
        // account <id> send <user> <msg>
        // account <id> status get
        // account <id> status set <status>
        // account <id> chat list
        // account <id> chat join <chat>
        // account <id> chat part <chat>
        // account <id> chat send <chat> <msg>
        // account <id> chat users <chat>
        // account <id> chat invite <chat> <user>
        "account" => parse_account_command(s),
        // buddy: <acc_id> status: <status> name: <name> alias: [alias]
        "buddy:" => parse_buddy(s),
        // message: <acc_id> <destination> <timestamp> <sender> <msg>
        "message:" => parse_message(s),
        // status: account <acc_id> status: <status>
        "status:" => parse_status(s),
        // chat: msg: <acc_id> <chat> <timestamp> <sender> <message>
        // chat: list: <acc_id> <chat_id> <chat_alias> <nick>
        // chat: user: <acc_id> <chat> <name> <alias> <state>
        "chat:" => parse_chat(s),
        // version
        "version" => Some(Message::Version),
        // bye
        "bye" => Some(Message::Bye),
        // quit
        "quit" => Some(Message::Quit),
        // help
        "help" => Some(Message::Help),
        _ => None,
    }
}

fn parse_message(s: Vec<&str>) -> Option<Message> {
    // message: <acc_id> <destination> <timestamp> <sender> <msg>
    if s.len() < 6 {
        return None;
    }
    Some(Message::Message {
        account_id: s[1].into(),
        destination: s[2].into(),
        timestamp: s[3].into(),
        sender: s[4].into(),
        message: s[5..].join(" "),
    })
}

fn parse_status(s: Vec<&str>) -> Option<Message> {
    // status: account <acc_id> status: <status>
    if s.len() < 5 {
        return None;
    }
    Some(Message::Status {
        account_id: s[2].into(),
        status: s[4].into(),
    })
}
fn parse_account(s: Vec<&str>) -> Option<Message> {
    // account: <id> <name> <protocol> <user> <status>
    if s.len() < 6 {
        return None;
    }
    Some(Message::Account {
        id: s[1].into(),
        name: s[2].into(),
        protocol: s[3].into(),
        user: s[4].into(),
        status: s[5].into(),
    })
}

fn parse_account_command(s: Vec<&str>) -> Option<Message> {
    // account list
    // account add <protocol> <user> <password>
    // account <id> delete
    // account <id> buddies [online]
    // account <id> collect
    // account <id> send <user> <msg>
    // account <id> status get
    // account <id> status set <status>
    // account <id> chat list
    // account <id> chat join <chat>
    // account <id> chat part <chat>
    // account <id> chat send <chat> <msg>
    // account <id> chat users <chat>
    // account <id> chat invite <chat> <user>

    if s.len() < 2 {
        return None;
    }

    // account list
    // account add <protocol> <user> <password>
    match s[1] {
        "list" => return Some(Message::AccountList),
        "add" => {
            if s.len() < 5 {
                return None;
            }
            return Some(Message::AccountAdd {
                protocol: s[2].into(),
                user: s[3].into(),
                password: s[4].into(),
            });
        }
        _ => (),
    }

    if s.len() < 3 {
        return None;
    }

    match s[2] {
        // account <id> delete
        "delete" => return Some(Message::AccountDelete { id: s[1].into() }),

        // account <id> buddies [online]
        "buddies" => {
            return Some(Message::BuddyList {
                account_id: s[1].into(),
                status: (*s.get(3).unwrap_or(&"")).into(),
            })
        }

        // account <id> collect
        "collect" => {
            return Some(Message::MessageCollect {
                account_id: s[1].into(),
            })
        }

        // account <id> send <user> <msg>
        "send" => {
            if s.len() < 5 {
                return None;
            }
            return Some(Message::MessageSend {
                account_id: s[1].into(),
                destination: s[3].into(),
                message: s[4..].join(" "),
            });
        }

        // account <id> status get
        // account <id> status set <status>
        "status" => {
            if s.len() < 4 {
                return None;
            }
            match s[3] {
                "get" => {
                    return Some(Message::StatusGet {
                        account_id: s[1].into(),
                    })
                }
                "set" => {
                    if s.len() < 5 {
                        return None;
                    }
                    return Some(Message::StatusSet {
                        account_id: s[1].into(),
                        status: s[4].into(),
                    });
                }
                _ => return None,
            }
        }

        // account <id> chat list
        // account <id> chat join <chat>
        // account <id> chat part <chat>
        // account <id> chat send <chat> <msg>
        // account <id> chat users <chat>
        // account <id> chat invite <chat> <user>
        "chat" => {
            if s.len() < 4 {
                return None;
            }
            match s[3] {
                "list" => {
                    return Some(Message::ChatList {
                        account_id: s[1].into(),
                    })
                }
                "join" => {
                    if s.len() < 5 {
                        return None;
                    }
                    return Some(Message::ChatJoin {
                        account_id: s[1].into(),
                        chat: s[4].into(),
                    });
                }
                "part" => {
                    if s.len() < 5 {
                        return None;
                    }
                    return Some(Message::ChatLeave {
                        account_id: s[1].into(),
                        chat: s[4].into(),
                    });
                }
                "send" => {
                    if s.len() < 6 {
                        return None;
                    }
                    return Some(Message::ChatMessageSend {
                        account_id: s[1].into(),
                        chat: s[4].into(),
                        message: s[5..].join(" "),
                    });
                }
                "users" => {
                    if s.len() < 5 {
                        return None;
                    }
                    return Some(Message::ChatUserList {
                        account_id: s[1].into(),
                        chat: s[4].into(),
                    });
                }
                "invite" => {
                    if s.len() < 6 {
                        return None;
                    }
                    return Some(Message::ChatUserInvite {
                        account_id: s[1].into(),
                        chat: s[4].into(),
                        user: s[5].into(),
                    });
                }
                _ => return None,
            }
        }

        _ => (),
    }

    None
}

fn parse_buddy(s: Vec<&str>) -> Option<Message> {
    // buddy: <acc_id> status: <status> name: <name> alias: [alias]
    if s.len() < 7 {
        return None;
    }
    Some(Message::Buddy {
        account_id: s[1].into(),
        status: s[3].into(),
        name: s[5].into(),
        alias: (*s.get(7).unwrap_or(&"")).into(),
    })
}

fn parse_chat(s: Vec<&str>) -> Option<Message> {
    // chat: msg: <acc_id> <chat> <timestamp> <sender> <message>
    // chat: list: <acc_id> <chat_id> <chat_alias> <nick>
    // chat: user: <acc_id> <chat> <name> <alias> <state>
    if s.len() < 6 {
        return None;
    }
    match s[1] {
        "msg:" => {
            if s.len() < 7 {
                return None;
            }
            Some(Message::ChatMessage {
                account_id: s[2].into(),
                chat: s[3].into(),
                timestamp: s[4].into(),
                sender: s[5].into(),
                message: s[6..].join(" "),
            })
        }
        "list:" => Some(Message::Chat {
            account_id: s[2].into(),
            chat: s[3].into(),
            alias: s[4].into(),
            nick: s[5].into(),
        }),
        "user:" => {
            if s.len() < 7 {
                return None;
            }
            Some(Message::ChatUser {
                account_id: s[2].into(),
                chat: s[3].into(),
                user: s[4].into(),
                alias: s[5].into(),
                status: s[6].into(),
            })
        }
        _ => None,
    }
}

fn parse_info(s: Vec<&str>) -> Option<Message> {
    // info: <msg>
    if s.len() < 2 {
        return None;
    }
    Some(Message::Info {
        message: s[1..].join(" "),
    })
}

impl From<String> for Message {
    fn from(s: String) -> Self {
        Message::Info { message: s }
    }
}

impl From<Message> for String {
    fn from(m: Message) -> Self {
        match m {
            Message::Info { message } => message,
            _ => "".into(),
        }
    }
}
