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

const getUserIdFromToken = (token) => {
    try {
        const base64Url = token.split('.')[1];
        const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
        const payload = JSON.parse(atob(base64));
        return payload.sub; // This is the UUID from auth.rs
    } catch (e) {
        return null;
    }
};

const acquireLock = async () => {
    const noteId = document.getElementById('noteId').value;
    const token = localStorage.getItem('token');
    
    if (!noteId) return; // New notes don't need a lock yet

    try {
        const response = await fetch(`http://localhost:3000/notes/${noteId}/lock`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${token}` }
        });

        if (response.status === 409) {
            // AppError::Conflict in your handlers.rs returns 409
            noteInput.disabled = true;
            statusDiv.innerText = "❌ Cette note est déjà en cours d'édition par quelqu'un d'autre.";
        } else if (response.ok) {
            statusDiv.innerText = "🔒 Verrouillé pour édition";
        }
    } catch (error) {
        console.error("Lock error:", error);
    }
};

// Trigger lock on focus
noteInput.addEventListener('focus', acquireLock);

const releaseLock = async () => {
    const id = noteId.value;
    if (!id) return;

    try {
        await fetch(`http://localhost:3000/notes/${id}/unlock`, {
            method: 'POST',
            headers: getAuthHeaders()
        });
        console.log("Note déverrouillée");
    } catch (err) {
        console.error("Erreur unlock:", err);
    }
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
    
    // We assume your backend now returns the full Note object
    notesList.innerHTML = notes.map(n => {
        // Simple logic: if I am not the owner, it might be read-only
        // (You can refine this if your backend sends a 'can_write' field)
        const isReadIndicator = n.locked_by ? "🔒" : ""; 
        
        return `
            <li class="note-item" onclick="editNote('${n.id}')">
                <span>
                    ${isReadIndicator} ${n.content.substring(0, 15)}...
                </span>
                <button onclick="event.stopPropagation(); promptShare('${n.id}')">🤝 Partager</button>
            </li>
        `;
    }).join('');
};

window.promptShare = async (id) => {
    const targetName = prompt("Username du destinataire :");
    if (!targetName) return;
    
    // Simple way to ask for permission level
    const canWrite = confirm("Autoriser la modification (OK = Oui / Annuler = Lecture seule) ?");

    try {
        const response = await fetch(`http://localhost:3000/notes/${id}/share`, {
            method: 'POST',
            headers: getAuthHeaders(),
            body: JSON.stringify({
                username: targetName,
                can_write: canWrite
            })
        });

        if (response.ok) {
            alert(`Note partagée en mode ${canWrite ? 'Lecture/Écriture' : 'Lecture seule'} !`);
        } else {
            const err = await response.json();
            alert("Erreur : " + err.error);
        }
    } catch (err) {
        console.error(err);
    }
};

const canUserWrite = (note, currentUserId) => {
    return note.owner_id === currentUserId || note.can_write === true;
};

window.editNote = async (id) => {
    const response = await fetch(`http://localhost:3000/notes/${id}`, {
        headers: getAuthHeaders()
    });
    const note = await response.json();

    noteId.value = note.id;
    noteInput.value = note.content;
    
    // Enter focus mode
    document.querySelector('.main-layout').classList.add('editing-active');
    
    // Check write permission and lock status
    if (!note.can_write) {
        setEditorState(true, "Lecture seule");
    } else if (note.locked_by && note.locked_by !== getUserIdFromToken(token)) {
        setEditorState(true, "Verrouillé par un tiers");
    } else {
        // Try to lock
        const lockRes = await fetch(`http://localhost:3000/notes/${id}/lock`, {
            method: 'POST',
            headers: getAuthHeaders()
        });
        setEditorState(!lockRes.ok, lockRes.ok ? "Enregistrer" : "Occupé...");
    }
    toggleUI(true);
};

function setEditorState(readOnly, btnText) {
    noteInput.readOnly = readOnly;
    saveBtn.innerText = btnText;
    saveBtn.disabled = readOnly;
}

// Helper to clean up UI state
const setReadOnlyMode = (isReadOnly, message = "Enregistrer") => {
    noteInput.readOnly = isReadOnly;
    saveBtn.disabled = isReadOnly;
    saveBtn.innerText = isReadOnly ? "🔒 Lecture Seule" : "Enregistrer";
    saveBtn.style.opacity = isReadOnly ? "0.5" : "1";
    if (message !== "Enregistrer") saveBtn.innerText = message;
};

document.getElementById('newBtn').addEventListener('click', async () => {
    await releaseLock(); // Release previous lock before clearing
    toggleUI(false);
    document.querySelector('.main-layout').classList.remove('editing-active');
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
    if (content === "") return alert("Le contenu ne peut pas être vide !");

    const id = noteId.value;
    const payload = {
        title: "Note", // Struct requires a title field
        content: content
    };

    try {
        let response;
        if (id) {
            // Update existing note
            response = await fetch(`http://localhost:3000/notes/${id}`, {
                method: 'PUT',
                headers: getAuthHeaders(),
                body: JSON.stringify(payload)
            });
        } else {
            // Create new note
            response = await fetch('http://localhost:3000/notes', {
                method: 'POST',
                headers: getAuthHeaders(),
                body: JSON.stringify(payload)
            });
        }

        if (response.ok) {
            const result = await response.json(); // Get the saved note back
            
            // If it was a new note, we now have an ID for it
            if (!id && result.id) {
                noteId.value = result.id;
            }

            alert("Sauvegardé !");
            
            // Optional: Don't call toggleUI(false) if you want to stay in the editor
            loadNotes(); // Refresh the list
        } else if (response.status === 409) {
            alert("Erreur : Cette note est verrouillée par un autre utilisateur.");
        } else {
            const errData = await response.json();
            alert("Erreur : " + (errData.error || "Action impossible"));
        }
    } catch (err) {
        console.error("Save error:", err);
        alert("Erreur de connexion au serveur.");
    }
};

document.getElementById('logoutBtn').addEventListener('click', () => {
    sessionStorage.removeItem('token');
    window.location.href = "register.html";
});
saveBtn.addEventListener('click', handleSave);
loadNotes();

window.addEventListener('beforeunload', () => {
    const id = noteId.value;
    const currentToken = sessionStorage.getItem('token');
    if (id && currentToken) {
        // Use keepalive to ensure the request finishes after the tab closes
        fetch(`http://localhost:3000/notes/${id}/unlock`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${currentToken}` },
            keepalive: true 
        });
    }
});