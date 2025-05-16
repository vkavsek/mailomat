-- create a seed user
INSERT INTO users (user_id, username, password_hash) 
VALUES ( 
  '6be0668f-b203-4bbc-b945-fafcf555610d',
  'admin',
  '$argon2id$v=19$m=19456,t=2,p=1$0vg2AKepQ+Su8CEUkTytKA$0a634i/lcjq9hu897tZ0h80kC4lXhxGkDAANt4mIBkM'
);
