# Development

-   Create a .env file in the root of the repository with 
    -   AUTH_SECRET (can be any string, used for signing cookies)
    -   DB_USERNAME
    -   DB_PASSWORD
-   Also change sql_util to point at a different postgres database endpoint, hopefully locally
-   Create your database with data/create.sql
-   Cargo run the project and visit localhost:8080
-   I am very happy to sit down with anybody for an hour and go over how the codebase is set up

# Deployment

-   I can deploy this lambda manually whenever changes are made
