use chrono::prelude::Utc;
use chrono::Timelike;
use serenity::async_trait;
use serenity::model::channel::{Message, ReactionType};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use whitelist::initialize_whitelist;

pub mod whitelist;

struct User {
    messages: u8,
}

struct Guild {
    users: HashMap<String, User>,
    emoji: ReactionType,
}

struct GlobalData {
    guilds: HashMap<String, Guild>,
    whitelist: HashMap<String, Vec<String>>,
    hour: u8,
}

impl TypeMapKey for GlobalData {
    type Value = Arc<RwLock<GlobalData>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        // Get guilds from cache
        let guild_ids = ready.guilds.iter().map(|g| g.id).collect::<Vec<_>>();

        for guild_id in guild_ids {
            let partial_guild = guild_id.to_partial_guild(&ctx.http).await;
            let guild = match partial_guild {
                Ok(guild) => guild,
                Err(_) => continue,
            };

            // Create emoji for reaction
            let emoji_name = match std::env::var("EMOJI_NAME") {
                Ok(emoji_name) => emoji_name,
                Err(_) => "zipper_mouth".to_string(),
            };

            let emoji_option = guild.emojis.iter().find(|e| e.1.name == emoji_name);
            let emoji = match emoji_option {
                Some(emoji_object) => ReactionType::Custom {
                    animated: emoji_object.1.animated,
                    id: emoji_object.1.id,
                    name: Some(emoji_object.1.name.clone()),
                },
                None => ReactionType::Unicode("🤐".to_string()),
            };

            // Add guild to guilds map
            let guild = Guild {
                users: HashMap::new(),
                emoji,
            };

            let data = ctx.data.write().await;

            let global_data = data
                .get::<GlobalData>()
                .expect("Expected GlobalData in TypeMap");

            global_data
                .write()
                .await
                .guilds
                .insert(guild_id.to_string(), guild);
        }
    }

    async fn guild_create(
        &self,
        ctx: Context,
        guild: serenity::model::guild::Guild,
        is_new: Option<bool>,
    ) {
        let guild_id = guild.id.to_string();

        let is_new: bool = match is_new {
            Some(is_new) => is_new,
            None => return,
        };

        if is_new {
            // Create emoji for reaction
            let emoji_name =
                std::env::var("EMOJI_NAME").expect("Expected an emoji name in the environment");

            let emoji_option = guild.emojis.iter().find(|e| e.1.name == emoji_name);
            let emoji = match emoji_option {
                Some(emoji_object) => ReactionType::Custom {
                    animated: emoji_object.1.animated,
                    id: emoji_object.1.id,
                    name: Some(emoji_object.1.name.clone()),
                },
                None => ReactionType::Unicode("🤐".to_string()),
            };

            // Add guild to guilds map
            let guild = Guild {
                users: HashMap::new(),
                emoji,
            };

            let data = ctx.data.write().await;

            let global_data = data
                .get::<GlobalData>()
                .expect("Expected GlobalData in TypeMap");

            global_data.write().await.guilds.insert(guild_id, guild);
        }
    }

    async fn guild_delete(
        &self,
        _ctx: Context,
        incomplete: serenity::model::guild::UnavailableGuild,
        _full: Option<serenity::model::guild::Guild>,
    ) {
        let guild_id = incomplete.id.to_string();

        // Remove guild from guilds map
        let data = _ctx.data.write().await;

        let global_data = data
            .get::<GlobalData>()
            .expect("Expected GlobalData in TypeMap");

        global_data.write().await.guilds.remove(&guild_id);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // If message is from bot, ignore it
        if msg.author.bot {
            return;
        }

        let data = ctx.data.read().await;
        let global_data = data
            .get::<GlobalData>()
            .expect("Expected GlobalData in TypeMap");

        // Reset guilds every hour
        if global_data.read().await.hour != Utc::now().hour() as u8 {
            for guild in global_data.write().await.guilds.values_mut() {
                guild.users.clear();
            }
            global_data.write().await.hour = Utc::now().hour() as u8;
        }

        let guild_id = match msg.guild_id {
            Some(guild_id) => guild_id.to_string(),
            None => return,
        };

        // If user is in whitelist, ignore message
        if global_data.read().await.whitelist.contains_key(&guild_id) {
            let whitelist = global_data.read().await.whitelist[&guild_id].clone();
            if whitelist.contains(&msg.author.id.to_string()) {
                return;
            }
        }

        let mut global_data_write = global_data.write().await;
        let guild = match global_data_write.guilds.get_mut(&guild_id) {
            Some(guild) => guild,
            None => return,
        };

        // Increment message count for user
        let user = guild
            .users
            .entry(msg.author.id.to_string())
            .or_insert(User { messages: 0 });
        user.messages += 1;

        // If user has sent enough messages, tell them to shut up
        let message_threshold = match std::env::var("MESSAGE_THRESHOLD") {
            Ok(message_threshold) => message_threshold.parse::<u8>().unwrap_or(50),
            Err(_) => 50,
        };

        if user.messages == message_threshold {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    "You talk too much, <@".to_string()
                        + &msg.author.id.to_string()
                        + ">, shut up!",
                )
                .await
                .ok();
        }

        // If author has sent more than enough messages, react with emoji from the server
        if user.messages > message_threshold {
            let _ = msg.react(&ctx.http, guild.emoji.clone()).await.ok();
        }
    }
}

#[tokio::main]
async fn main() {
    // Get bot token from environment
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Initialize whitelist
    let whitelist = initialize_whitelist().await;

    // Set gateway intents
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    // Create instance of bot
    let mut bot = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating instance of bot");

    // Set global data
    {
        let mut data = bot.data.write().await;

        let global_data = GlobalData {
            guilds: HashMap::new(),
            whitelist,
            hour: Utc::now().hour() as u8,
        };

        data.insert::<GlobalData>(Arc::new(RwLock::new(global_data)));
    }

    // Start bot shard
    if let Err(start_error) = bot.start().await {
        println!(
            "An error occurred while running the client: {:?}",
            start_error
        );
    }
}
