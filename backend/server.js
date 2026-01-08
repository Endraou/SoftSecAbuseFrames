const express = require('express');
const cors = require('cors');
const sqlite3 = require('sqlite3').verbose();
const app = express();
const port = 3000;

app.use(cors());
app.use(express.json());

const db = new sqlite3.Database('./database.db', (err) => {
    if (err) console.error(err.message);
    console.log('Connecté à la base SQL');
});

db.run(`CREATE TABLE IF NOT EXISTS notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT,
    created_at TEXT
)`);

// Route unique pour Créer OU Modifier
app.post('/notes', (req, res) => {
    const { id, text, date } = req.body;

    if (!text || text.trim() === "") {
        return res.status(400).json({ error: "Le texte est vide" });
    }

    if (id) {
        // UPDATE : on modifie la note existante
        const sql = `UPDATE notes SET content = ?, created_at = ? WHERE id = ?`;
        db.run(sql, [text, date, id], function (err) {
            if (err) return res.status(500).json({ error: err.message });
            res.json({ message: "Note mise à jour  !" });
        });
    } else {
        // INSERT : on crée une nouvelle note
        const sql = `INSERT INTO notes (content, created_at) VALUES (?, ?)`;
        db.run(sql, [text, date], function (err) {
            if (err) return res.status(500).json({ error: err.message });
            res.status(201).json({ message: "Note créée !", id: this.lastID });
        });
    }
});

app.get('/notes', (req, res) => {
    const sql = `SELECT * FROM notes ORDER BY created_at DESC`;
    db.all(sql, [], (err, rows) => {
        if (err) return res.status(500).json({ error: err.message });
        res.json(rows);
    });
});

app.delete('/notes/:id', (req, res) => {
    const id = req.params.id;
    const sql = `DELETE FROM notes WHERE id = ?`;
    db.run(sql, id, function (err) {
        if (err) return res.status(500).json({ error: err.message });
        res.json({ message: "Note supprimée !" });
    });
});

app.listen(port, () => {
    console.log(`Serveur prêt sur http://localhost:${port}`);
});