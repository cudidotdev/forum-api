# (API) Forum Application

This is the front-end of a Forum application built with  rust, actix-web, tokio, postgresql.

In this application, you can 
- Write, save and comment on a post.
- Reply a comment, which in turn be replied to.
- Filter post by hashtags and sort by latest or highest engaged post.
- View users and see their created post.
- View posts saved by a user.

Here is a preview:
![forum_homepage](https://github.com/CudiLala/Forum-App/assets/88282186/c73b9345-ef06-4831-88d0-74603bfcb0fc)

## Setup
Here, I assume you may already have rust, cargo and postgres installed and accessible from your command line.

To set up the application locally, you first clone this repository and modify your environment variables.
There is a file called `.env.example` which you can rename to `.env` and use as your environment variables. 
Below are the bash codes for the above 

``` bash
# clone the repo
git clone https://github.com/CudiLala/forum-api.git

# copy the example env as your .env file
cp .env.example .env
```

The `.env.exmaple` file looks like this

```env
THREADS = 8
JWT_SECRET = 'yourjwtsecret'

PG.USER = 'forum'
PG.PASSWORD = 'postgresuserpassword'
PG.HOST = '127.0.0.1'
PG.PORT = '5432'
PG.DBNAME = 'forum'
PG.POOL.MAX_SIZE = '16'

CORS_ORIGIN = 'http://localhost:5173'
SERVER_PORT = 8080
```

In your `.env` file, you can edit `PG.PASSWORD` field. But it's better to leave the `PG.USER` and `PG.DBNAME` as given. If you edited `PG.PASSWORD` make sure to edit the `user.sql` file before proceeding with the postgres setup. The `schema.sql` file uses the user `forum`, so you can edit all that too, if you wish to change the user.

### Set up postgres
First, cd into the `forum-api` directory.

To set up postgresql, first we create the postgres user and forum database. If you edited `PG.PASSWORD` update `user.sql` file with your new password
```bash
  psql -f user.sql -U postgres -W 
```
This may produce an error, if you do not have password authentication set up of user 'postgres'.
If it fails, you can copy the content of `user.sql` and paste in your postgres shell, in pgadmin or any of your postgres IDE. Or you can switch to your postgres user and run the above code without `-W` flag.

Next, you will have to create postgres shema. To do that, run
```bash
  psql -f schema.sql -U forum -W 
```
You will have to enter the passowrd you created for user forum.
This may produce an error, if you do not have password authentication set up for any user.
If it fails, you can copy the content of `schema.sql` and paste in your postgres shell, in pgadmin or any of your postgres IDE.

You can then build your rust binaries with 
```bash
  # development
  cargo build

  #production
  cargo build --release
```

You can then run your application
```bash
  cargo run
```

If you encountered any error setting up the application you can contact me @ augustinemadu9@gmail.com
