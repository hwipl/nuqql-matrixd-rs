#[derive(Debug, Eq, PartialEq)]
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
    // account <acc_id> chat list
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

impl std::str::FromStr for Message {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s).ok_or(())
    }
}

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

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Message::Info { message } => write!(f, "info: {message}\r\n"),
            Message::Account {
                id,
                name,
                protocol,
                user,
                status,
            } => write!(f, "account: {id} {name} {protocol} {user} {status}\r\n"),
            Message::AccountList => write!(f, "account list\r\n"),
            Message::AccountAdd {
                protocol,
                user,
                password,
            } => write!(f, "account add {protocol} {user} {password}\r\n"),
            Message::AccountDelete { id } => write!(f, "account {id} delete\r\n"),
            Message::Buddy {
                account_id,
                status,
                name,
                alias,
            } => write!(
                f,
                // TODO: alias OK like this?
                "buddy: {account_id} status: {status} name: {name} alias: {alias}\r\n"
            ),
            Message::BuddyList { account_id, status } => {
                // TODO: status OK like this?
                write! {f, "account {account_id} buddies {status}\r\n"}
            }
            Message::Message {
                account_id,
                destination,
                timestamp,
                sender,
                message,
            } => write!(
                f,
                "message: {account_id} {destination} {timestamp} {sender} {message}\r\n"
            ),
            Message::MessageCollect { account_id } => write!(f, "account {account_id} collect\r\n"),
            Message::MessageSend {
                account_id,
                destination,
                message,
            } => write!(f, "account {account_id} send {destination} {message}\r\n"),
            Message::Status { account_id, status } => {
                write!(f, "status: account {account_id} status: {status}\r\n")
            }
            Message::StatusGet { account_id } => write!(f, "account {account_id} status get\r\n"),
            Message::StatusSet { account_id, status } => {
                write!(f, "account {account_id} status set {status}\r\n")
            }
            Message::Chat {
                account_id,
                chat,
                alias,
                nick,
            } => write!(f, "chat: list: {account_id} {chat} {alias} {nick}\r\n"),
            Message::ChatList { account_id } => write!(f, "account {account_id} chat list\r\n"),
            Message::ChatJoin { account_id, chat } => {
                write!(f, "account {account_id} chat join {chat}\r\n")
            }
            Message::ChatLeave { account_id, chat } => {
                write!(f, "account {account_id} chat part {chat}\r\n")
            }
            Message::ChatMessage {
                account_id,
                chat,
                timestamp,
                sender,
                message,
            } => write!(
                f,
                "chat: msg: {account_id} {chat} {timestamp} {sender} {message}\r\n"
            ),
            Message::ChatMessageSend {
                account_id,
                chat,
                message,
            } => write!(f, "account {account_id} chat send {chat} {message}\r\n"),
            Message::ChatUser {
                account_id,
                chat,
                user,
                alias,
                status,
            } => write!(
                f,
                "chat: user: {account_id} {chat} {user} {alias} {status}\r\n"
            ),
            Message::ChatUserList { account_id, chat } => {
                write!(f, "account {account_id} chat users {chat}\r\n")
            }
            Message::ChatUserInvite {
                account_id,
                chat,
                user,
            } => write!(f, "account {account_id} chat invite {chat} {user}\r\n"),
            Message::Version => write!(f, "version\r\n"),
            Message::Bye => write!(f, "bye\r\n"),
            Message::Quit => write!(f, "quit\r\n"),
            Message::Help => write!(f, "help\r\n"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message() {
        for msg in vec![
            Message::Info {
                message: "hello".into(),
            },
            Message::Account {
                id: "1".into(),
                name: "matrix".into(),
                protocol: "matrix".into(),
                user: "user".into(),
                status: "online".into(),
            },
            Message::AccountList,
            Message::AccountAdd {
                protocol: "matrix".into(),
                user: "user".into(),
                password: "password".into(),
            },
            Message::AccountDelete { id: "1".into() },
            Message::Buddy {
                account_id: "1".into(),
                status: "online".into(),
                name: "user".into(),
                alias: "".into(),
            },
            Message::Buddy {
                account_id: "1".into(),
                status: "online".into(),
                name: "user".into(),
                alias: "alias".into(),
            },
            Message::BuddyList {
                account_id: "1".into(),
                status: "".into(),
            },
            Message::BuddyList {
                account_id: "1".into(),
                status: "online".into(),
            },
            Message::Message {
                account_id: "1".into(),
                destination: "other_user".into(),
                timestamp: "1700000000".into(),
                sender: "me".into(),
                message: "".into(),
            },
            Message::Message {
                account_id: "1".into(),
                destination: "other_user".into(),
                timestamp: "1700000000".into(),
                sender: "me".into(),
                message: "this is a test message\ndoes it work?\n \n \n  -test".into(),
            },
            Message::MessageCollect {
                account_id: "1".into(),
            },
            Message::MessageSend {
                account_id: "1".into(),
                destination: "other_user".into(),
                message: "".into(),
            },
            Message::MessageSend {
                account_id: "1".into(),
                destination: "other_user".into(),
                message: "this is a test message\ndoes it work?\n \n \n  -test".into(),
            },
            Message::Status {
                account_id: "1".into(),
                status: "online".into(),
            },
            Message::StatusGet {
                account_id: "1".into(),
            },
            Message::StatusSet {
                account_id: "1".into(),
                status: "online".into(),
            },
            Message::Chat {
                account_id: "1".into(),
                chat: "some_chat".into(),
                alias: "chat_alias".into(),
                nick: "my_name".into(),
            },
            Message::ChatList {
                account_id: "1".into(),
            },
            Message::ChatJoin {
                account_id: "1".into(),
                chat: "some_chat".into(),
            },
            Message::ChatLeave {
                account_id: "1".into(),
                chat: "some_chat".into(),
            },
            Message::ChatMessage {
                account_id: "1".into(),
                chat: "some_chat".into(),
                timestamp: "1700000000".into(),
                sender: "some_user".into(),
                message: "".into(),
            },
            Message::ChatMessage {
                account_id: "1".into(),
                chat: "some_chat".into(),
                timestamp: "1700000000".into(),
                sender: "some_user".into(),
                message: "this is a test message\ndoes it work?\n \n \n  -test".into(),
            },
            Message::ChatMessageSend {
                account_id: "1".into(),
                chat: "some_chat".into(),
                message: "".into(),
            },
            Message::ChatMessageSend {
                account_id: "1".into(),
                chat: "some_chat".into(),
                message: "this is a test message\ndoes it work?\n \n \n  -test".into(),
            },
            Message::ChatUser {
                account_id: "1".into(),
                chat: "some_chat".into(),
                user: "user".into(),
                alias: "alias".into(),
                status: "online".into(),
            },
            Message::ChatUserList {
                account_id: "1".into(),
                chat: "some_chat".into(),
            },
            Message::ChatUserInvite {
                account_id: "1".into(),
                chat: "some_chat".into(),
                user: "user".into(),
            },
            Message::Version,
            Message::Bye,
            Message::Quit,
            Message::Help,
        ] {
            assert_eq!(msg, msg.to_string().parse().unwrap());
        }
    }
}
