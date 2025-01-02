# Development

-   Keep in mind that this is a _very_ rough repo that was created in 30 minutes by ripping out all of the necessary code from elsewhere
-   Start by changing a bunch of hardcoded values
    -   This will hopefully change at some point but this was just ripped out of a different repository
    -   Change the secret values in sql_util and main (should be the things that use get_secret)
    -   Also change sql_util to point at a different postgres database endpoint, hopefully locally
    -   Change dynamodb to point at different tables
-   Cargo run the project and visit localhost:8080
-   I am very happy to sit down with anybody for an hour and go over how the codebase is set up

# Deployment

-   I can deploy this lambda manually whenever changes are made
