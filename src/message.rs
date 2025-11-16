enum Message {
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
