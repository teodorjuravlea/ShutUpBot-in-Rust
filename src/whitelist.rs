use std::collections::HashMap;
use std::fs;
use std::io::Write;

pub async fn initialize_whitelist() -> HashMap<String, Vec<String>> {
    /*
    // If whitelist file does not exist, create it
    if !std::path::Path::new("whitelist.txt").exists() {
        fs::File::create("whitelist.txt").expect("Error creating file");
    }

    // Read whitelist from file
    let contents = fs::read_to_string("whitelist.txt").expect("Error reading file");

    let mut whitelist: HashMap<String, Vec<String>> = HashMap::new();

    let lines: Vec<&str> = contents.split('\n').collect();
    for line in lines {
        let parts: Vec<&str> = line.split(' ').collect();
        let guild_id = parts[0];
        let user_id = parts[1];
        let guild = whitelist.entry(guild_id.to_string()).or_default();
        guild.push(user_id.to_string());
    }


    whitelist
    */
    HashMap::new()
}

pub async fn add_user_to_whitelist(guild_id: String, user_id: String) {
    /*
    // Read whitelist from file
    let contents = fs::read_to_string("whitelist.txt").expect("Error reading file");

    let mut whitelist: HashMap<String, Vec<String>> = HashMap::new();

    let lines: Vec<&str> = contents.split('\n').collect();
    for line in lines {
        let parts: Vec<&str> = line.split(' ').collect();
        let guild_id = parts[0];
        let user_id = parts[1];
        let guild = whitelist.entry(guild_id.to_string()).or_default();
        guild.push(user_id.to_string());
    }

    let guild = whitelist.entry(guild_id).or_default();
    guild.push(user_id);

    // Write updated whitelist to file
    let mut file = fs::File::create("whitelist.txt").expect("Error creating file");
    for (guild_id, user_ids) in whitelist {
        for user_id in user_ids {
            let line = format!("{} {}\n", guild_id, user_id);
            file.write_all(line.as_bytes())
                .expect("Error writing to file");
        }
    }
    */
}
