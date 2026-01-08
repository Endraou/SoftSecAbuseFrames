const notesList = document.getElementById('notesList');
const noteInput = document.getElementById('noteInput');
const noteId = document.getElementById('noteId');
const deleteBtn = document.getElementById('deleteBtn');
const saveBtn = document.getElementById('saveBtn'); // On garde cet ID fixe

const toggleUI = (isEditing) => {
    if (isEditing) {
        deleteBtn.style.display = "inline-block";
        saveBtn.innerText = "Mettre à jour";
        saveBtn.style.backgroundColor = "#ff9800"; // Petit changement de couleur pour l'edit
    } else {
        deleteBtn.style.display = "none";
        saveBtn.innerText = "Sauvegarder";
        saveBtn.style.backgroundColor = "#03dac6";
        noteId.value = "";
        noteInput.value = "";
    }
};

const loadNotes = async () => {
    try {
        const response = await fetch('http://localhost:3000/notes');
        const notes = await response.json();
        notesList.innerHTML = notes.map(n => `
            <li class="note-item" onclick="editNote(${n.id}, '${n.content.replace(/'/g, "\\'")}')">
                <span>${n.content.substring(0, 15)}...</span>
            </li>
        `).join('');
    } catch (err) {
        console.error("Erreur chargement :", err);
    }
};

window.editNote = (id, content) => {
    noteId.value = id;
    noteInput.value = content;
    toggleUI(true);
};

document.getElementById('newBtn').addEventListener('click', () => {
    toggleUI(false);
});

// La fonction de suppression
const deleteNote = async (id) => {
    if (!confirm("Supprimer cette note ?")) return;
    try {
        await fetch(`http://localhost:3000/notes/${id}`, { method: 'DELETE' });
        toggleUI(false);
        loadNotes();
    } catch (err) {
        console.error("Erreur suppression :", err);
    }
};

deleteBtn.addEventListener('click', () => {
    if (noteId.value) deleteNote(noteId.value);
});

// La fonction unique pour sauvegarder OU mettre à jour
const handleSave = async () => {
    const content = noteInput.value.trim();
    if (content === "") return alert("C'est vide !");

    const id = noteId.value; // Si vide = création, si rempli = update

    try {
        const response = await fetch('http://localhost:3000/notes', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                id: id ? parseInt(id) : null,
                text: content,
                date: new Date().toISOString()
            })
        });

        if (response.ok) {
            toggleUI(false);
            loadNotes();
        }
    } catch (err) {
        console.error("Erreur save/update :", err);
    }
};

saveBtn.addEventListener('click', handleSave);
loadNotes();