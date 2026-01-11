const notesList = document.getElementById('notesList');
const noteInput = document.getElementById('noteInput');
const noteId = document.getElementById('noteId');
const deleteBtn = document.getElementById('deleteBtn');
const saveBtn = document.getElementById('saveBtn'); // On garde cet ID fixe

// Fonction pour décoder le JWT et vérifier l'expiration (tiré de index.js)
const isTokenExpired = (token) => {
    if (!token) return true;
    try {
        const base64Url = token.split('.')[1];
        const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
        const jsonPayload = decodeURIComponent(atob(base64).split('').map(c =>
            '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2)
        ).join(''));
        const { exp } = JSON.parse(jsonPayload);
        return (Date.now() >= exp * 1000);
    } catch (e) {
        return true;
    }
};

// Vérification immédiate au chargement
const token = sessionStorage.getItem('token');

if (!token || isTokenExpired(token)) {
    console.log("Session invalide ou absente, redirection...");
    sessionStorage.removeItem('token');
    window.location.href = "register.html";
} else {
    console.log("Token trouvé, accès autorisé miaou");
}

const getAuthHeaders = () => {
    const currentToken = sessionStorage.getItem('token');
    return {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${currentToken}`
    };
};

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
    const response = await fetch('http://localhost:3000/notes', {
        headers: getAuthHeaders()
    });
    const notes = await response.json();
    notesList.innerHTML = notes.map(n => `
        <li class="note-item">
            <span onclick="editNote('${n.id}', '${n.content.replace(/'/g, "\\'")}')">
                ${n.content.substring(0, 15)}...
            </span>
            <button onclick="promptShare('${n.id}')">🤝 Partager</button>
        </li>
    `).join('');
};

window.promptShare = async (id) => {
    const targetName = prompt("Username du destinataire :");
    if (!targetName) return;

    try {
        const response = await fetch(`http://localhost:3000/notes/${id}/share`, {
            method: 'POST',
            headers: getAuthHeaders(),
            body: JSON.stringify({
                username: targetName,
                can_write: true
            })
        });

        if (response.ok) {
            alert("Accès partagé miaou");
        } else {
            const err = await response.json();
            alert("Erreur : " + err.error);
        }
    } catch (err) {
        console.error(err);
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
    if (!id) return;
    if (!confirm("Supprimer cette note définitivement ?")) return;

    try {
        const response = await fetch(`http://localhost:3000/notes/${id}`, {
            method: 'DELETE', // Vérifie que ton main.rs autorise DELETE dans le CORS
            headers: getAuthHeaders()
        });

        if (response.ok) {
            toggleUI(false);
            loadNotes();
        } else {
            alert("Action non autorisée ou note introuvable.");
        }
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

    const id = noteId.value; // C'est l'UUID de la note sélectionnée

    const payload = {
        title: "Note mise à jour", // Le backend Rust attend un titre
        content: content
    };

    try {
        let response;

        if (id) {
            // MODE UPDATE : On vise l'URL avec l'ID et la méthode PUT
            response = await fetch(`http://localhost:3000/notes/${id}`, {
                method: 'PUT',
                headers: getAuthHeaders(),
                body: JSON.stringify(payload)
            });
        } else {
            // MODE CREATION : On utilise POST sur la route racine
            response = await fetch('http://localhost:3000/notes', {
                method: 'POST',
                headers: getAuthHeaders(),
                body: JSON.stringify(payload)
            });
        }

        if (response.ok) {
            toggleUI(false); // Reset le formulaire
            loadNotes();     // Rafraîchit la liste
        } else {
            const error = await response.json();
            console.error("Erreur serveur :", error);
        }
    } catch (err) {
        console.error("Erreur lors de la sauvegarde :", err);
    }
};

document.getElementById('logoutBtn').addEventListener('click', () => {
    sessionStorage.removeItem('token');
    window.location.href = "register.html";
});
saveBtn.addEventListener('click', handleSave);
loadNotes();