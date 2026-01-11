#![allow(dead_code)]

fn hardcoded_api_keys() {
    // Should trigger DE0703 - hardcoded secret
    let stripe_key = "sk_live_51H1234567890abcdef";
    
    // Should trigger DE0703 - hardcoded secret
    let github_token = "ghp_1234567890abcdefghijklmnopqrstuv";
    
    // Should trigger DE0703 - hardcoded secret
    let aws_key = "AKIAIOSFODNN7EXAMPLE";
}

fn hardcoded_passwords() {
    // Should trigger DE0703 - hardcoded secret
    let password = "password=MySecretP@ssw0rd123";
    
    // Should trigger DE0703 - hardcoded secret
    let api_key = "api_key=abc123def456ghi789jkl";
    
    // Should trigger DE0703 - hardcoded secret
    let secret = "secret=TopSecretValue2024!";
}

fn high_entropy_strings() {
    // Should trigger DE0703 - hardcoded secret
    let token = "AbC123XyZ789MnO456PqR";
}

fn more_api_keys() {
    // Should trigger DE0703 - hardcoded secret
    let google_key = "AIzaSyDaGmWKa4JsXZ-HjGw7ISLn_3namBGewQe";
    
    // Should trigger DE0703 - hardcoded secret
    let google_oauth = "ya29.a0AfH6SMBx8gK9...";
}

fn credentials_in_strings() {
    // Should trigger DE0703 - hardcoded secret
    let cred = "credential=MySecureToken123";
    
    // Should trigger DE0703 - hardcoded secret
    let auth = "auth=Bearer_Token_12345";
}

fn main() {}
