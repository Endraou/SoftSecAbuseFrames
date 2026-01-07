// On cible les éléments du DOM
const noteInput = document.getElementById('noteInput');
const saveBtn = document.getElementById('saveBtn');
const logoutBtn = document.getElementById('logoutBtn');
const statusDiv = document.getElementById('status'); // Targeted once here for efficiency

// Logout Functionality
if (logoutBtn) {
    logoutBtn.addEventListener('click', () => {
        console.log("log out");
        localStorage.removeItem('token');
        window.location.href = "register.html";
    });
}

// Function to check if the token is expired
const isTokenExpired = (token) => {
    if (!token) return true;
    try {
        // JWTs are: header.payload.signature
        const base64Url = token.split('.')[1]; 
        const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
        const jsonPayload = decodeURIComponent(atob(base64).split('').map(function(c) {
            return '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2);
        }).join(''));

        const { exp } = JSON.parse(jsonPayload);
        // exp is in seconds, Date.now() is in milliseconds
        return (Date.now() >= exp * 1000);
    } catch (e) {
        return true; // If parsing fails, treat as expired
    }
};

// Check every 10 seconds
setInterval(() => {
    const token = localStorage.getItem('token');
    
    if (isTokenExpired(token)) {
        console.log("Session expired. Logging out...");
        localStorage.removeItem('token');
        window.location.href = 'register.html';
    }
}, 10000); // 10,000 milliseconds = 10 seconds

// Fonction pour simuler l'envoi vers un backend
const sendToBackend = async (content) => {
    const token = localStorage.getItem('token');
    
    if (!token || isTokenExpired(token)) {
        alert("Session expirée. Veuillez vous reconnecter.");
        window.location.href = "register.html";
        return;
    }

    statusDiv.innerText = "Saving...";
    statusDiv.style.color = "white";

    const payload = {
        title: "New Note",
        content: content
    };

    try {
        const response = await fetch('http://localhost:3000/notes', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify(payload)
        });

        if (response.ok) {
            statusDiv.innerText = "✅ Note saved successfully!";
            noteInput.value = ""; // ONLY clear here if successful
        } else {
            statusDiv.innerText = "❌ Error: Could not save note.";
        }
    } catch (error) {
        statusDiv.innerText = "❌ Server is offline.";
    }
};

saveBtn.addEventListener('click', () => {
    const content = noteInput.value.trim();
    if (content !== "") {
        sendToBackend(content);
        // Removed noteInput.value = "" from here
    } else {
        statusDiv.innerText = "⚠️ La note est vide !";
        statusDiv.style.color = "#cf6679";
    }
});