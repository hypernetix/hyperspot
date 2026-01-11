// Test file for DE0407: hardcoded connection strings (should trigger)

fn setup_database() {
    // Should trigger DE0407 - hardcoded connection
    let _pg = "postgres://user:password@localhost:5432/mydb";

    // Should trigger DE0407 - hardcoded connection
    let _mysql = "mysql://root:secret@127.0.0.1:3306/app";

    // Should trigger DE0407 - hardcoded connection
    let _mongo = "mongodb://admin:pass@localhost:27017/test";

    // Should trigger DE0407 - hardcoded connection
    let _redis = "redis://localhost:6379/0";

    // Should trigger DE0407 - hardcoded connection
    let _amqp = "amqp://guest:guest@localhost:5672/";
}

fn main() {
    setup_database();
}
