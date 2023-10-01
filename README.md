#(API) Forum Application

This is the front-end of a Forum application built with  rust, actix-web, tokio, postgresql.

In this application, you can 
- You can write, save and comment on a post.
- You can reply a comment, which in turn be replied to.
- You can filter post by hashtags and sort by latest or highest engaged post.
- You can also view users and see their created post.
- You can also see posts saved by a user.

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

PG.USER = 'postgresuser'
PG.PASSWORD = 'postgresuserpassword'
PG.HOST = '127.0.0.1'
PG.PORT = '5432'
PG.DBNAME = 'forum'
PG.POOL.MAX_SIZE = '16'

CORS_ORIGIN = 'http://localhost:5173'
SERVER_PORT = 8080
```

