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

CREATE TYPE color AS ENUM ('green', 'red', 'blue', 'yellow', 'violet', 'purple');

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
)
