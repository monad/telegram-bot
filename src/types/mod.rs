//! Types of the Telegram API.
//!
//! This module contains definitions of the types defined
//! [here](https://core.telegram.org/bots/api/#available-types). Many Telegram
//! types, like "Location", map directly to Rust-structs. Other Telegram types,
//! like "Message", was made more rusty by using enums.
//!
//! All types implement `Decodable` and `Encodable`, so they can be serialized
//! as JSON. Non existing JSON-fields will result in `None` values for `Option`
//! types. `None` values don't result in JSON fields.
//!

use rustc_serialize::{Decodable, Encodable, Decoder, Encoder};
use std::convert::Into;
use std::fmt;

// ===========================================================================
// Helpers
// ===========================================================================
// Macro to implement "Encodable" quickly. "None" fields won't be encoded.
macro_rules! impl_encode {
    (
        $ty:ty, $count:expr,
        [$($id:expr => $field:ident),*],
        [$($o_id:expr => $o_field:ident),*]
    ) => {
        impl Encodable for $ty {
            fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
                e.emit_struct(stringify!($ty), $count, |e| {
                    $(
                        // Irritating special case.
                        if stringify!($field) == "_type" {
                            try!(e.emit_struct_field("type", $id, |e| {
                                self.$field.encode(e)
                            }));
                        }

                        else {                        
                            try!(e.emit_struct_field(stringify!($field), $id, |e| {
                                self.$field.encode(e)
                            }));
                        }
                    )*
                    $(
                        if let Some(ref v) = self.$o_field {
                            try!(e.emit_struct_field(
                                stringify!($o_field), $o_id, |e| {
                                v.encode(e)
                            }));
                        }
                    )*

                    Ok(())
                })
            }
        }
    }
}

// Calls impl_encode for each subtype of InputMessageContent.
macro_rules! encode_subtypes {
    (
        $iqr:ident, $count:expr,
        [$($id:expr => $field:ident),*],
        [$($o_id:expr => $o_field:ident),*]
    ) => {
        impl_encode!($iqr<InputTextMessageContent>, $count,
                     [$( $id => $field ), *],
                     [$( $o_id => $o_field ), *]);
        impl_encode!($iqr<InputLocationMessageContent>, $count,
                     [$( $id => $field ), *],
                     [$( $o_id => $o_field ), *]);
        impl_encode!($iqr<InputVenueMessageContent>, $count,
                     [$( $id => $field ), *],
                     [$( $o_id => $o_field ), *]);
        impl_encode!($iqr<InputContactMessageContent>, $count,
                     [$( $id => $field ), *],
                     [$( $o_id => $o_field ), *]);
    }
}

// Decodes a field with a given name. If successful: Return decoded
// value. If not: Exit function with error value.
macro_rules! try_field {
    ($d:ident, $name:expr) => {
        try!($d.read_struct_field($name, 0, Decodable::decode))
    }
}


// ===========================================================================
// Telegram primitive types
// ===========================================================================
/// The Telegram "Integer": Currently i64.
pub type Integer = i64;
/// The Telegram "Float": Currently f32.
pub type Float = f32;


// ===========================================================================
// Types not explicitly mentioned or somehow different from Telegram types
// ===========================================================================
/// All API responses are from this type. Mostly used internal.
#[derive(RustcDecodable, Debug, PartialEq, Clone)]
pub struct Response<T: Decodable> {
    pub ok: bool,
    pub error_code: Option<Integer>,
    pub description: Option<String>,
    pub result: Option<T>,
}

// ---------------------------------------------------------------------------
/// Represents one of "ReplyKeyboardMarkup", "ReplyKeyboardHide" or
/// "ForceReply". Used for the "reply_markup" field.
#[derive(Debug, PartialEq, Clone)]
pub enum ReplyMarkup {
    Keyboard(ReplyKeyboardMarkup),
    /// The boolean corresponds to the "selective" field of "ReplyKeyboardHide"
    KeyboardHide(bool),
    /// The boolean corresponds to the "selective" field of "ForceReply"
    ForceReply(bool),
}

impl From<ReplyKeyboardMarkup> for ReplyMarkup {
    fn from(keyboard: ReplyKeyboardMarkup) -> ReplyMarkup {
        ReplyMarkup::Keyboard(keyboard)
    }
}

impl Encodable for ReplyMarkup {
    fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
        match *self {
            ReplyMarkup::Keyboard(ref k) => k.encode(e),
            ReplyMarkup::KeyboardHide(b) => {
                e.emit_struct("ReplyKeyboardHide", 2, |e| {
                    try!(e.emit_struct_field("hide_keyboard", 0, |e| {
                        true.encode(e)
                    }));
                    e.emit_struct_field("selective", 1, |e| b.encode(e))
                })
            },
            ReplyMarkup::ForceReply(b) => {
                e.emit_struct("ForceReply", 2, |e| {
                    try!(e.emit_struct_field("force_reply", 0, |e| {
                        true.encode(e)
                    }));
                    e.emit_struct_field("selective", 1, |e| b.encode(e))
                })
            },
        }
    }
}

// ---------------------------------------------------------------------------
/// Strongly typed ChatAction. Instead of passing a String to the
/// `send_chat_action` method, this is used.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ChatAction {
    Typing,
    UploadPhoto,
    RecordVideo,
    UploadVideo,
    RecordAudio,
    UploadAudio,
    UploadDocument,
    FindLocation,
}

impl Decodable for ChatAction {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        Ok(match &*try!(d.read_str()) {
            "typing" => ChatAction::Typing,
            "upload_photo" => ChatAction::UploadPhoto,
            "record_video" => ChatAction::RecordVideo,
            "upload_video" => ChatAction::UploadVideo,
            "record_audio" => ChatAction::RecordAudio,
            "upload_audio" => ChatAction::UploadAudio,
            "upload_document" => ChatAction::UploadDocument,
            "find_location" => ChatAction::FindLocation,
            _ => return Err(d.error("Not a valid chat action")),
        })
    }
}

impl Into<&'static str> for ChatAction {
    fn into(self) -> &'static str {
        match self {
             ChatAction::Typing => "typing",
             ChatAction::UploadPhoto => "upload_photo",
             ChatAction::RecordVideo => "record_video",
             ChatAction::UploadVideo => "upload_video",
             ChatAction::RecordAudio => "record_audio",
             ChatAction::UploadAudio => "upload_audio",
             ChatAction::UploadDocument => "upload_document",
             ChatAction::FindLocation => "find_location",
        }
    }
}

impl ToString for ChatAction {
    fn to_string(&self) -> String {
        Into::<&str>::into(*self).into()
    }
}

impl Encodable for ChatAction {
    fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
        e.emit_str((*self).into())
    }
}

// ---------------------------------------------------------------------------
/// Either a Private or a Group or a Channel. Used in "chat" field of Message. Has some
/// useful methods for less typing.
#[derive(Debug, PartialEq, Clone)]
pub enum Chat {
    Private {
        id: Integer,
        first_name: String,
        last_name: Option<String>,
        username: Option<String>,
    },
    Group {
        id: Integer,
        title: String,
        is_supergroup: bool
    },
    Channel {
        id: Integer,
        title: String,
        name: Option<String>
    },
}

impl Chat {
    /// Returns the chat id, which is needed to send messages.
    pub fn id(&self) -> Integer {
        match self {
            &Chat::Private { id, .. } => id,
            &Chat::Group { id, .. } => id,
            &Chat::Channel { id, .. } => id,
        }
    }

    /// Returns if the Chat is a User
    pub fn is_user(&self) -> bool {
        if let &Chat::Private {..} = self { true } else { false }
    }

    /// Returns if the Chat is a Group
    pub fn is_group(&self) -> bool {
        if let &Chat::Group { is_supergroup, .. } = self { !is_supergroup } else { false }
    }

    /// Returns if the Chat is a SuperGroup
    pub fn is_supergroup(&self) -> bool {
        if let &Chat::Group { is_supergroup, .. } = self { is_supergroup } else { false }
    }

    /// Returns if the Chat is a Channel
    pub fn is_channel(&self) -> bool {
        if let &Chat::Channel {..} = self { true } else { false }
    }

    pub fn to_user(&self) -> Option<User> {
        if let &Chat::Private { id, ref first_name, ref last_name, ref username } = self {
            Some(User {
                id: id,
                first_name: first_name.clone(),
                last_name: last_name.clone(),
                username: username.clone(),
            })
        } else { None }
    }
}

impl Decodable for Chat {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("Chat", 0, |d| {
            // All kinds of chat have an 'id' and a 'type' fields
            let id : Integer = try_field!(d, "id");
            let typ: String = try_field!(d, "type");

            match typ.as_ref() {
                "private" => {
                    Ok(Chat::Private {
                        id: id,
                        first_name: try_field!(d, "first_name"),
                        last_name: try_field!(d, "last_name"),
                        username: try_field!(d, "username"),
                    })
                }
                "group" => {
                    Ok(Chat::Group {
                        id: id,
                        title: try_field!(d, "title"),
                        is_supergroup: false
                    })
                }
                "supergroup" => {
                    Ok(Chat::Group {
                        id: id,
                        title: try_field!(d, "title"),
                        is_supergroup: true
                    })
                }
                "channel" => {
                    Ok(Chat::Channel {
                        id: id,
                        title: try_field!(d, "title"),
                        name: try_field!(d, "username"),
                    })
                }
                _ => Err(d.error(&format!("Invalid chat type: {}", typ)))
            }
        })
    }
}

impl Encodable for Chat {
    fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {

        match self {
            &Chat::Private { id, ref first_name, ref last_name, ref username } => {
                e.emit_struct("Chat", 5, |e| {
                    try!(e.emit_struct_field("id", 0, |e| {
                        id.encode(e)
                    }));
                    try!(e.emit_struct_field("type", 1, |e| {
                        "private".encode(e)
                    }));
                    try!(e.emit_struct_field("first_name", 2, |e| {
                        first_name.encode(e)
                    }));
                    try!(e.emit_struct_field("last_name", 3, |e| {
                        last_name.encode(e)
                    }));
                    try!(e.emit_struct_field("type", 4, |e| {
                        username.encode(e)
                    }));
                    Ok(())
                })
            },
            &Chat::Group { id, ref title, is_supergroup} => {
                e.emit_struct("Chat", 3, |e| {
                    try!(e.emit_struct_field("id", 0, |e| {
                        id.encode(e)
                    }));
                    try!(e.emit_struct_field("type", 1, |e| {
                        let typ = if is_supergroup { "supergroup" } else { "group" };
                        typ.encode(e)
                    }));
                    try!(e.emit_struct_field("title", 2, |e| {
                        title.encode(e)
                    }));
                    Ok(())
                })
            },
            &Chat::Channel { id, ref title, ref name} => {
                e.emit_struct("Channel", 3, |e| {
                    try!(e.emit_struct_field("id", 0, |e| {
                        id.encode(e)
                    }));
                    try!(e.emit_struct_field("title", 1, |e| {
                        title.encode(e)
                    }));
                    try!(e.emit_struct_field("username", 2, |e| {
                        name.encode(e)
                    }));
                    Ok(())
                })
            },
        }
    }
}

// ---------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub message_id: Integer,
    pub from: User,
    pub chat: Chat,
    pub date: Integer,

    // forward_from and forward_date in one
    pub forward: Option<(User, Integer)>,
    pub reply: Option<Box<Message>>,

    pub msg: MessageType,

    pub caption: Option<String>,
}

// We need to implement this on our own, because the field "msg" is not a real
// JSON field.
impl Decodable for Message {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("Message", 0, |d| {
            let maybe_forward_from = try!(d.read_struct_field("forward_from", 0, Decodable::decode));
            let maybe_forward_date = try!(d.read_struct_field("forward_date", 0, Decodable::decode));
            let maybe_forward = match (maybe_forward_from, maybe_forward_date) {
                (Some(from), Some(date)) => Some((from, date)),
                _ => None,
            };
            Ok(Message {
                message_id: try_field!(d, "message_id"),
                from: try_field!(d, "from"),
                chat: try_field!(d, "chat"),
                date: try_field!(d, "date"),
                forward: maybe_forward,
                reply: try_field!(d, "reply_to_message"),
                msg: try!(MessageType::decode(d)),
                caption: try_field!(d, "caption"),
            })
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum MessageType {
    Text(String),
    Audio(Audio),
    Voice(Voice),
    File(Document),
    Photo(Vec<PhotoSize>),
    Sticker(Sticker),
    Video(Video),
    Contact(Contact),
    Location(Location),
    NewChatParticipant(User),
    LeftChatParticipant(User),
    NewChatTitle(String),
    NewChatPhoto(Vec<PhotoSize>),
    DeleteChatPhoto,
    GroupChatCreated,
    SuperGroupChatCreated(GroupToSuperGroupMigration),
    ChannelChatCreated,
}

impl Decodable for MessageType {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        // Tries to decode a field with the given name. If the field does NOT
        // exist: Does nothing. If the field does exist: Return the decoded
        // value. If any other decoder error occured: Return the error.
        macro_rules! maybe_field {
            ($d:ident, $name:expr, $variant:ident) => {{
                if let Some(val) = try!($d.read_struct_field(
                    $name, 0, Decodable::decode)) {
                    return Ok(MessageType::$variant(val));
                };
            }}
        }

        // There is always just one of these fields used, so we can infer the
        // enum variant from it.
        // These are the message types that carry additional data
        maybe_field!(d, "text", Text);
        maybe_field!(d, "audio", Audio);
        maybe_field!(d, "voice", Voice);
        maybe_field!(d, "file", File);
        maybe_field!(d, "photo", Photo);
        maybe_field!(d, "document", File);
        maybe_field!(d, "sticker", Sticker);
        maybe_field!(d, "video", Video);
        maybe_field!(d, "contact", Contact);
        maybe_field!(d, "location", Location);
        maybe_field!(d, "new_chat_participant", NewChatParticipant);
        maybe_field!(d, "left_chat_participant", LeftChatParticipant);
        maybe_field!(d, "new_chat_title", NewChatTitle);
        maybe_field!(d, "new_chat_photo", NewChatPhoto);

        // Message types without additional data
        if let Some(true) = try!(d.read_struct_field(
            "delete_chat_photo", 0, Decodable::decode)) {
            return Ok(MessageType::DeleteChatPhoto);
        };
        if let Some(true) = try!(d.read_struct_field(
            "group_chat_created", 0, Decodable::decode)) {
            return Ok(MessageType::GroupChatCreated);
        };

        if let Some(true) = try!(d.read_struct_field(
            "supergroup_chat_created", 0, Decodable::decode)) {
            return Ok(MessageType::SuperGroupChatCreated(GroupToSuperGroupMigration {
                from: try_field!(d, "migrate_from_chat_id"),
                to: try_field!(d, "migrate_to_chat_id"),
            }))
        };

        if let Some(true) = try!(d.read_struct_field(
            "channel_chat_created", 0, Decodable::decode)) {
            return Ok(MessageType::ChannelChatCreated);
        };

        // None of the tested fields is present: This is an error
        Err(d.error("No field for inferring message type is set"))
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GroupToSuperGroupMigration {
    pub from: Integer,
    pub to: Integer,
}

// ---------------------------------------------------------------------------
/// Strongly typed ParseMode. Instead of passing a String to the
/// `send_message` method, this is used.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseMode {
    Markdown,
    Html,
}

impl fmt::Display for ParseMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            ParseMode::Markdown => "Markdown",
            ParseMode::Html => "HTML",
        })
    }
}

// ===========================================================================
// Telegram types directly mapped to Rust types
// ===========================================================================
/// Telegram type "User" (directly mapped)
#[derive(RustcDecodable, Debug, PartialEq, Clone)]
pub struct User {
    pub id: Integer,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

impl_encode!(User, 4,
    [0 => id, 1 => first_name],
    [2 => last_name, 3 => username]);

// ---------------------------------------------------------------------------
/// Telegram type "PhotoSize" (directly mapped)
#[derive(RustcDecodable, Debug, PartialEq, Clone)]
pub struct PhotoSize {
    pub file_id: String,
    pub width: Integer,
    pub height: Integer,
    pub file_size: Option<Integer>,
}

impl_encode!(PhotoSize, 4,
    [0 => file_id, 1 => width, 2 => height],
    [3 => file_size]);

// ---------------------------------------------------------------------------
/// Telegram type "Audio" (directly mapped)
#[derive(RustcDecodable, Debug, PartialEq, Clone)]
pub struct Audio {
    pub file_id: String,
    pub duration: Integer,
    pub performer: Option<String>,
    pub title: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<Integer>,
}

impl_encode!(Audio, 6,
             [0 => file_id, 1 => duration],
             [2 => performer, 3 => title,
              4 => mime_type, 5 => file_size]);

// ---------------------------------------------------------------------------
/// Telegram type "Voice" (directly mapped)
#[derive(RustcDecodable, Debug, PartialEq, Clone)]
pub struct Voice {
    pub file_id: String,
    pub duration: Integer,
    pub mime_type: Option<String>,
    pub file_size: Option<Integer>,
}

impl_encode!(Voice, 4,
             [0 => file_id, 1 => duration],
             [2 => mime_type, 3 => file_size]);


// ---------------------------------------------------------------------------
/// Telegram type "Document" (directly mapped)
#[derive(RustcDecodable, Debug, PartialEq, Clone)]
pub struct Document {
    pub file_id: String,
    pub thumb: Option<PhotoSize>,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<Integer>,
}

impl_encode!(Document, 5,
    [0 => file_id, 1 => thumb],
    [2 => file_name, 3 => mime_type, 4 => file_size]);

// ---------------------------------------------------------------------------
/// Telegram type "Sticker" (directly mapped)
#[derive(RustcDecodable, Debug, PartialEq, Clone)]
pub struct Sticker {
    pub file_id: String,
    pub width: Integer,
    pub height: Integer,
    pub thumb: Option<PhotoSize>,
    pub file_size: Option<Integer>,
}

impl_encode!(Sticker, 5,
    [0 => file_id, 1 => width, 2 => height, 3 => thumb],
    [4 => file_size]);

// ---------------------------------------------------------------------------
/// Telegram type "Video" (directly mapped)
#[derive(RustcDecodable, Debug, PartialEq, Clone)]
pub struct Video {
    pub file_id: String,
    pub width: Integer,
    pub height: Integer,
    pub duration: Integer,
    pub thumb: Option<PhotoSize>,
    pub mime_type: Option<String>,
    pub file_size: Option<Integer>,
}

impl_encode!(Video, 7,
    [0 => file_id, 1 => width, 2 => height, 3 => duration, 4 => thumb],
    [5 => mime_type, 6 => file_size]);

// ---------------------------------------------------------------------------
/// Telegram type "Contact" (directly mapped)
#[derive(RustcDecodable, Debug, PartialEq, Clone)]
pub struct Contact {
    pub phone_number: String,
    pub first_name: String,
    pub last_name: Option<String>,
    pub user_id: Option<Integer>,
}

impl_encode!(Contact, 4,
    [0 => phone_number, 1 => first_name],
    [2 => last_name, 3 => user_id]);

// ---------------------------------------------------------------------------
/// Telegram type "Location" (directly mapped)
#[derive(RustcDecodable, RustcEncodable, Debug, PartialEq, Clone, Copy)]
pub struct Location {
    pub longitude: Float,
    pub latitude: Float,
}

// ---------------------------------------------------------------------------
/// Telegram type "Update" (directly mapped)
#[derive(RustcDecodable, Debug, PartialEq, Clone)]
pub struct Update {
    pub update_id: Integer,
    pub message: Option<Message>,
    pub inline_query: Option<InlineQuery>
}

// impl_encode!(Update, 2,
//     [0 => update_id],
//     [1 => message]);

// ---------------------------------------------------------------------------
/// Telegram type "UserProfilePhotos" (directly mapped)
#[derive(RustcDecodable, RustcEncodable, Debug, PartialEq, Clone)]
pub struct UserProfilePhotos {
    pub total_count: Integer,
    pub photos: Vec<Vec<PhotoSize>>,
}

// ---------------------------------------------------------------------------
/// Telegram type "ReplyKeyboardMarkup" (directly mapped)
#[derive(RustcDecodable, Debug, PartialEq, Clone)]
pub struct ReplyKeyboardMarkup {
    pub keyboard: Vec<Vec<String>>,
    pub resize_keyboard: Option<bool>,
    pub one_time_keyboard: Option<bool>,
    pub selective: Option<bool>,
}

impl Default for ReplyKeyboardMarkup {
    fn default() -> Self {
        ReplyKeyboardMarkup {
            keyboard: Vec::new(),
            resize_keyboard: None,
            one_time_keyboard: None,
            selective: None,
        }
    }
}

impl_encode!(ReplyKeyboardMarkup, 4,
    [0 => keyboard],
    [1 => resize_keyboard, 2 => one_time_keyboard, 3 => selective]);

// ---------------------------------------------------------------------------
#[derive(RustcDecodable, Debug, PartialEq, Clone)]
pub struct InlineQuery {
    pub id: String,
    pub from: User,
    pub location: Option<Location>,
    pub query: String,
    pub offset: String
}

// ---------------------------------------------------------------------------
/// Simulate subtyping by using a trait InlineQueryResult to constrain
/// the various types of Result available.
pub trait InlineQueryResult {}

// ---------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct InlineQueryResultArticle<K: InputMessageContent> {
    pub _type: String,
    pub id: String,
    pub title: String,
    pub input_message_content: K,
    pub reply_markup: Option<InlineKeyboardMarkup>,
    pub url: Option<String>,
    pub hide_url: Option<bool>,
    pub description: Option<String>,
    pub thumb_url: Option<String>,
    pub thumb_width: Option<Integer>,
    pub thumb_height: Option<Integer>
}

impl<T: InputMessageContent> InlineQueryResult for InlineQueryResultArticle<T> {}

encode_subtypes!(InlineQueryResultArticle, 11,
                 [0 => _type, 1 => id, 2 => title, 3 => input_message_content],
                 [4 => reply_markup, 5 => url, 6 => hide_url, 7 => description,
                  8 => thumb_url, 9 => thumb_width, 10 => thumb_height]);

// ---------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct InlineQueryResultPhoto<K: InputMessageContent> {
    pub _type: String,
    pub id: String,
    pub photo_url: String,
    pub thumb_url: String,
    pub photo_width: Option<Integer>,
    pub photo_height: Option<Integer>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub caption: Option<String>,
    pub reply_markup: Option<InlineKeyboardMarkup>,
    pub input_message_content: Option<K>
}

impl<T: InputMessageContent> InlineQueryResult for InlineQueryResultPhoto<T> {}

encode_subtypes!(InlineQueryResultPhoto, 12,
                 [0 => _type, 1 => id, 2 => photo_url, 3 => thumb_url],
                 [4 => photo_width, 5 => photo_height, 6 => title,
                  7 => description, 8 => caption, 9 => reply_markup,
                  10 => input_message_content]);
// ---------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct InlineQueryResultGif<K: InputMessageContent> {
    pub _type: String,
    pub id: String,
    pub gif_url: String,
    pub gif_width: Option<Integer>,
    pub gif_height: Option<Integer>,
    pub thumb_url: String,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub reply_markup: Option<InlineKeyboardMarkup>,
    pub input_message_content: Option<K>
}

impl<T: InputMessageContent> InlineQueryResult for InlineQueryResultGif<T> {}

encode_subtypes!(InlineQueryResultGif, 10,
                 [0 => _type, 1 => id, 2 => gif_url, 5 => thumb_url],
                 [3 => gif_width, 4 => gif_height, 6 => title, 7 => caption,
                  8 => reply_markup, 9 => input_message_content]);

// ---------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct InlineQueryResultMpeg4Gif<K: InputMessageContent> {
    pub _type: String,
    pub id: String,
    pub mpeg4_url: String,
    pub mpeg4_width: Option<Integer>,
    pub mpeg4_height: Option<Integer>,
    pub thumb_url: String,
    pub title: Option<String>,
    pub caption: Option<String>,
    pub reply_markup: Option<InlineKeyboardMarkup>,
    pub input_message_content: Option<K>
}

impl<T: InputMessageContent> InlineQueryResult for InlineQueryResultMpeg4Gif<T> {}

encode_subtypes!(InlineQueryResultMpeg4Gif, 10,
                 [0 => _type, 1 => id, 2 => mpeg4_url, 5 => thumb_url],
                 [3 => mpeg4_width, 4 => mpeg4_height, 6 => title,
                  7 => caption, 8 => reply_markup, 9 => input_message_content]);

// ---------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct InlineQueryResultVideo<K: InputMessageContent> {
    pub _type: String,
    pub id: String,
    pub video_url: String,
    pub mime_type: String,
    pub thumb_url: String,
    pub title: String,
    pub caption: Option<String>,
    pub video_width: Option<Integer>,
    pub video_height: Option<Integer>,
    pub video_duration: Option<Integer>,
    pub description: Option<String>,
    pub reply_markup: Option<InlineKeyboardMarkup>,
    pub input_message_content: Option<K>
}

impl<T: InputMessageContent> InlineQueryResult for InlineQueryResultVideo<T> {}

encode_subtypes!(InlineQueryResultVideo, 13,
                 [0 => _type, 1 => id, 2 => video_url, 3 => mime_type,
                  4 => thumb_url, 5 => title],
                 [6 => caption, 7 => video_width, 8 => video_height,
                  9 => video_duration, 10 => description, 11 => reply_markup,
                  12 => input_message_content]);

// ---------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct InlineQueryResultAudio<K: InputMessageContent> {
    pub _type: String,
    pub id: String,
    pub audio_url: String,
    pub title: String,
    pub performer: Option<String>,
    pub audio_duration: Option<Integer>,
    pub reply_markup: Option<InlineKeyboardMarkup>,
    pub input_message_content: Option<K>
}

impl<T: InputMessageContent> InlineQueryResult for InlineQueryResultAudio<T> {}

encode_subtypes!(InlineQueryResultAudio, 8,
                 [0 => _type, 1 => id, 2 => audio_url, 3 => title],
                 [4 => performer, 5 => audio_duration, 6 => reply_markup,
                  7 => input_message_content]);

// ---------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct InlineQueryResultVoice<K: InputMessageContent> {
    pub _type: String,
    pub id: String,
    pub voice_url: String,
    pub title: String,
    pub voice_duration: Option<Integer>,
    pub reply_markup: Option<InlineKeyboardMarkup>,
    pub input_message_content: Option<K>
}

impl<T: InputMessageContent> InlineQueryResult for InlineQueryResultVoice<T> {}

encode_subtypes!(InlineQueryResultVoice, 7,
                 [0 => _type, 1 => id, 2 => voice_url, 3 => title],
                 [4 => voice_duration, 5 => reply_markup,
                  6 => input_message_content]);

// ---------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct InlineQueryResultDocument<K: InputMessageContent> {
    pub _type: String,
    pub id: String,
    pub title: String,
    pub caption: Option<String>,
    pub document_url: String,
    pub mime_type: String,
    pub description: Option<String>,
    pub reply_markup: Option<InlineKeyboardMarkup>,
    pub input_message_content: Option<K>,
    pub thumb_url: Option<String>,
    pub thumb_width: Option<Integer>,
    pub thumb_height: Option<Integer>
}

impl<T: InputMessageContent> InlineQueryResult for InlineQueryResultDocument<T> {}

encode_subtypes!(InlineQueryResultDocument, 12,
                 [0 => _type, 1 => id, 2 => title, 4 => document_url,
                  5 => mime_type],
                 [3 => caption, 6 => description, 7 => reply_markup,
                  8 => input_message_content, 9 => thumb_url,
                  10 => thumb_width, 11 => thumb_height]);

// ---------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct InlineQueryResultLocation<K: InputMessageContent> {
    pub _type: String,
    pub id: String,
    pub latitude: Float,
    pub longitude: Float,
    pub title: String,
    pub reply_markup: Option<InlineKeyboardMarkup>,
    pub input_message_content: Option<K>,
    pub thumb_url: Option<String>,
    pub thumb_width: Option<Integer>,
    pub thumb_height: Option<Integer>
}

impl<T: InputMessageContent> InlineQueryResult for InlineQueryResultLocation<T> {}

encode_subtypes!(InlineQueryResultLocation, 10,
                 [0 => _type, 1 => id, 2 => latitude, 3 => longitude,
                  4 => title],
                 [5 => reply_markup, 6 => input_message_content, 7 => thumb_url,
                  8 => thumb_width, 9 => thumb_height]);

// ---------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct InlineQueryResultVenue<K: InputMessageContent> {
    pub _type: String,
    pub id: String,
    pub latitude: Float,
    pub longitude: Float,
    pub title: String,
    pub address: String,
    pub foursquare_id: Option<String>,
    pub reply_markup: Option<InlineKeyboardMarkup>,
    pub input_message_content: Option<K>,
    pub thumb_url: Option<String>,
    pub thumb_width: Option<Integer>,
    pub thumb_height: Option<Integer>
}

impl<T: InputMessageContent> InlineQueryResult for InlineQueryResultVenue<T> {}

encode_subtypes!(InlineQueryResultVenue, 12,
                 [0 => _type, 1 => id, 2 => latitude, 3 => longitude,
                  4 => title, 5 => address],
                 [6 => foursquare_id, 7 => reply_markup, 8 => input_message_content,
                  9 => thumb_url, 10 => thumb_width, 11 => thumb_height]);

// ---------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub struct InlineQueryResultContact<K: InputMessageContent> {
    pub _type: String,
    pub id: String,
    pub phone_number: String,
    pub first_name: String,
    pub last_name: Option<String>,
    pub reply_markup: Option<InlineKeyboardMarkup>,
    pub input_message_content: Option<K>,
    pub thumb_url: Option<String>,
    pub thumb_width: Option<Integer>,
    pub thumb_height: Option<Integer>
}

encode_subtypes!(InlineQueryResultContact, 10,
                 [0 => _type, 1 => id, 2 => phone_number, 3 => first_name],
                 [4 => last_name, 5 => reply_markup, 6 => input_message_content,
                  7 => thumb_url, 8 => thumb_width, 9 => thumb_height]);

// ---------------------------------------------------------------------------
#[derive(RustcEncodable, Debug, PartialEq, Clone)]
pub struct InlineKeyboardButton {
    pub text: String,
    pub url: Option<String>,
    pub callback_data: Option<String>,
    pub switch_inline_query: Option<String>
}

#[derive(RustcEncodable, Debug, PartialEq, Clone)]
pub struct InlineKeyboardMarkup {
    pub inline_keyboard: Vec<Vec<InlineKeyboardButton>>
}

// ---------------------------------------------------------------------------
/// InputMessageContent and 'subtypes'.
pub trait InputMessageContent {}

#[derive(Debug, PartialEq, Clone)]
pub struct InputTextMessageContent {
    pub message_text: String,
    pub parse_mode: Option<String>,
    pub disable_web_page_preview: Option<bool>
}

impl_encode!(InputTextMessageContent, 3, [0 => message_text],
             [1 => parse_mode, 2 => disable_web_page_preview]);
             
impl InputMessageContent for InputTextMessageContent {}

#[derive(RustcEncodable, Debug, PartialEq, Clone)]
pub struct InputLocationMessageContent {
    pub latitude: Float,
    pub longitude: Float
}

impl InputMessageContent for InputLocationMessageContent {}
                     
#[derive(RustcEncodable, Debug, PartialEq, Clone)]
pub struct InputVenueMessageContent {
    pub latitude: Float,
    pub longitude: Float,
    pub title: String,
    pub address: String,
    pub foursquare_id: Option<String>
}

impl InputMessageContent for InputVenueMessageContent {}

#[derive(RustcEncodable, Debug, PartialEq, Clone)]
pub struct InputContactMessageContent {
    pub phone_number: String,
    pub first_name: String,
    pub last_name: Option<String>
}

impl InputMessageContent for InputContactMessageContent {}

// ===========================================================================
// Unit tests (mainly encode & decode)
// ===========================================================================
mod test;
