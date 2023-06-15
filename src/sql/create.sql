CREATE TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  username VARCHAR(50) NOT NULL,
  password_hash VARCHAR(200) NOT NULL,
  created_at TIMESTAMP NOT NULL
);

CREATE UNIQUE INDEX username_lower_unique_index ON users (LOWER(username));

CREATE TABLE IF NOT EXISTS posts (
  id SERIAL PRIMARY KEY,
  title VARCHAR(100) NOT NULL,
  body VARCHAR(1000) NOT NULL,
  user_id INT NOT NULL,
  created_at TIMESTAMP NOT NULL,

  FOREIGN KEY (user_id)
    REFERENCES users(id)
    ON DELETE CASCADE
);

CREATE TYPE color AS ENUM ('green', 'red', 'blue', 'yellow', 'purple');

CREATE TABLE IF NOT EXISTS topics (
  id SERIAL PRIMARY KEY,
  name VARCHAR(50) UNIQUE NOT NULL,
  color color NOT NULL,
  created_at TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS posts_topics_relationship (
  post_id INT NOT NULL,
  topic_id INT NOT NULL,

  PRIMARY KEY(post_id, topic_id),

  FOREIGN KEY (post_id) 
    REFERENCES posts(id)
    ON DELETE CASCADE,

  FOREIGN KEY (topic_id)
    REFERENCES topics(id)
    ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS saved_posts (
  user_id INT NOT NULL,
  post_id INT NOT NULL,

  PRIMARY KEY(user_id, post_id),

  FOREIGN KEY (user_id) 
    REFERENCES users(id)
    ON DELETE CASCADE,

  FOREIGN KEY (post_id)
    REFERENCES posts(id)
    ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS post_comments (
  id SERIAL PRIMARY KEY,
  body VARCHAR(500) NOT NULL,
  post_id INT NOT NULL,
  user_id INT NOT NULL,
  comment_id INT,
  created_at TIMESTAMP NOT NULL,

  FOREIGN KEY (post_id)
    REFERENCES posts(id)
    ON DELETE CASCADE,

  FOREIGN KEY (comment_id)
    REFERENCES post_comments(id)
    ON DELETE CASCADE
)