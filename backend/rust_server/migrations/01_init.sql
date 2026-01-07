-- Users Table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL
);

-- Notes Table
CREATE TABLE notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    -- Locking mechanism for write access
    locked_by UUID REFERENCES users(id),
    locked_at TIMESTAMP WITH TIME ZONE
);

-- Permissions Table (Sharing)
CREATE TABLE note_shares (
    note_id UUID REFERENCES notes(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    can_write BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (note_id, user_id)
);