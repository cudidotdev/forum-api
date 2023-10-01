--
-- PostgreSQL database dump
--

-- Dumped from database version 16.0
-- Dumped by pg_dump version 16.0

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: color; Type: TYPE; Schema: public; Owner: forum
--

CREATE TYPE public.color AS ENUM (
    'green',
    'red',
    'blue',
    'yellow',
    'purple'
);


ALTER TYPE public.color OWNER TO forum;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: hashtags; Type: TABLE; Schema: public; Owner: forum
--

CREATE TABLE public.hashtags (
    id integer NOT NULL,
    name character varying(50) NOT NULL,
    color public.color NOT NULL,
    created_at timestamp without time zone NOT NULL
);


ALTER TABLE public.hashtags OWNER TO forum;

--
-- Name: post_comments; Type: TABLE; Schema: public; Owner: forum
--

CREATE TABLE public.post_comments (
    id integer NOT NULL,
    body character varying(500) NOT NULL,
    post_id integer NOT NULL,
    user_id integer NOT NULL,
    comment_id integer,
    created_at timestamp without time zone NOT NULL
);


ALTER TABLE public.post_comments OWNER TO forum;

--
-- Name: post_comments_id_seq; Type: SEQUENCE; Schema: public; Owner: forum
--

CREATE SEQUENCE public.post_comments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.post_comments_id_seq OWNER TO forum;

--
-- Name: post_comments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: forum
--

ALTER SEQUENCE public.post_comments_id_seq OWNED BY public.post_comments.id;


--
-- Name: posts; Type: TABLE; Schema: public; Owner: forum
--

CREATE TABLE public.posts (
    id integer NOT NULL,
    title character varying(100) NOT NULL,
    body character varying(1000) NOT NULL,
    user_id integer NOT NULL,
    created_at timestamp without time zone NOT NULL
);


ALTER TABLE public.posts OWNER TO forum;

--
-- Name: posts_hashtags_relationship; Type: TABLE; Schema: public; Owner: forum
--

CREATE TABLE public.posts_hashtags_relationship (
    post_id integer NOT NULL,
    hashtag_id integer NOT NULL
);


ALTER TABLE public.posts_hashtags_relationship OWNER TO forum;

--
-- Name: posts_id_seq; Type: SEQUENCE; Schema: public; Owner: forum
--

CREATE SEQUENCE public.posts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.posts_id_seq OWNER TO forum;

--
-- Name: posts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: forum
--

ALTER SEQUENCE public.posts_id_seq OWNED BY public.posts.id;


--
-- Name: saved_posts; Type: TABLE; Schema: public; Owner: forum
--

CREATE TABLE public.saved_posts (
    user_id integer NOT NULL,
    post_id integer NOT NULL
);


ALTER TABLE public.saved_posts OWNER TO forum;

--
-- Name: topics_id_seq; Type: SEQUENCE; Schema: public; Owner: forum
--

CREATE SEQUENCE public.topics_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.topics_id_seq OWNER TO forum;

--
-- Name: topics_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: forum
--

ALTER SEQUENCE public.topics_id_seq OWNED BY public.hashtags.id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: forum
--

CREATE TABLE public.users (
    id integer NOT NULL,
    username character varying(50) NOT NULL,
    password_hash character varying(200) NOT NULL,
    created_at timestamp without time zone NOT NULL
);


ALTER TABLE public.users OWNER TO forum;

--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: forum
--

CREATE SEQUENCE public.users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.users_id_seq OWNER TO forum;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: forum
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: hashtags id; Type: DEFAULT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.hashtags ALTER COLUMN id SET DEFAULT nextval('public.topics_id_seq'::regclass);


--
-- Name: post_comments id; Type: DEFAULT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.post_comments ALTER COLUMN id SET DEFAULT nextval('public.post_comments_id_seq'::regclass);


--
-- Name: posts id; Type: DEFAULT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.posts ALTER COLUMN id SET DEFAULT nextval('public.posts_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: post_comments post_comments_pkey; Type: CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.post_comments
    ADD CONSTRAINT post_comments_pkey PRIMARY KEY (id);


--
-- Name: posts posts_pkey; Type: CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_pkey PRIMARY KEY (id);


--
-- Name: posts_hashtags_relationship posts_topics_relationship_pkey; Type: CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.posts_hashtags_relationship
    ADD CONSTRAINT posts_topics_relationship_pkey PRIMARY KEY (post_id, hashtag_id);


--
-- Name: saved_posts saved_posts_pkey; Type: CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.saved_posts
    ADD CONSTRAINT saved_posts_pkey PRIMARY KEY (user_id, post_id);


--
-- Name: hashtags topics_name_key; Type: CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.hashtags
    ADD CONSTRAINT topics_name_key UNIQUE (name);


--
-- Name: hashtags topics_pkey; Type: CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.hashtags
    ADD CONSTRAINT topics_pkey PRIMARY KEY (id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: username_lower_unique_index; Type: INDEX; Schema: public; Owner: forum
--

CREATE UNIQUE INDEX username_lower_unique_index ON public.users USING btree (lower((username)::text));


--
-- Name: post_comments post_comments_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.post_comments
    ADD CONSTRAINT post_comments_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.post_comments(id) ON DELETE CASCADE;


--
-- Name: post_comments post_comments_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.post_comments
    ADD CONSTRAINT post_comments_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON DELETE CASCADE;


--
-- Name: posts_hashtags_relationship posts_topics_relationship_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.posts_hashtags_relationship
    ADD CONSTRAINT posts_topics_relationship_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON DELETE CASCADE;


--
-- Name: posts_hashtags_relationship posts_topics_relationship_topic_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.posts_hashtags_relationship
    ADD CONSTRAINT posts_topics_relationship_topic_id_fkey FOREIGN KEY (hashtag_id) REFERENCES public.hashtags(id) ON DELETE CASCADE;


--
-- Name: posts posts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.posts
    ADD CONSTRAINT posts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: saved_posts saved_posts_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.saved_posts
    ADD CONSTRAINT saved_posts_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.posts(id) ON DELETE CASCADE;


--
-- Name: saved_posts saved_posts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: forum
--

ALTER TABLE ONLY public.saved_posts
    ADD CONSTRAINT saved_posts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

