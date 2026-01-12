const statusDiv = document.getElementById('status');
sessionStorage.removeItem('token');

let attempts = 0;
let isLocked = false;
const MAX_ATTEMPTS = 5;
const LOCK_TIME = 30000;

function lockInterface() {
    isLocked = true;
    const buttons = [document.getElementById('loginBtn'), document.getElementById('regBtn')];
    
    buttons.forEach(btn => btn.disabled = true);
    
    let timeLeft = LOCK_TIME / 1000;
    const timer = setInterval(() => {
        showMessage(`Trop d'essais. Attendez ${timeLeft}s...`);
        timeLeft--;
        
        if (timeLeft < 0) {
            clearInterval(timer);
            isLocked = false;
            attempts = 0;
            buttons.forEach(btn => btn.disabled = false);
            showMessage("Vous pouvez réessayer.", false);
        }
    }, 1000);
}

// Helper to show messages in the UI instead of alerts
const showMessage = (msg, isError = true) => {
    statusDiv.innerText = msg;
    statusDiv.style.color = isError ? "#cf6679" : "#03dac6"; // Red for error, Teal for success
    statusDiv.style.marginTop = "10px";
};
        
async function handleAuth(url, data) {
    if (isLocked) return;

    showMessage("Processing...", false);

    try {
        const response = await fetch(url, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });

        const result = await response.json();

        if (response.ok) {
            sessionStorage.setItem('token', result.token);
            showMessage("Success! Redirecting...", false);
            setTimeout(() => {
                window.location.href = "index.html";
            }, 1000);
        } else {
            // On compte l'échec ici
            attempts++;
            if (attempts >= MAX_ATTEMPTS) {
                lockInterface();
            } else {
                showMessage(result.message || "Authentication failed.");
            }
        }
    } catch (error) {
        showMessage("Connection failed. Is the server running?");
    }
};

document.getElementById('regBtn').onclick = () => {
    const user = document.getElementById('username').value;
    const pass = document.getElementById('password').value;

    if (user.trim() === "" || pass.trim() === "") {
        showMessage("Username and password are required.");
        return; // <--- THIS is what stops the code from calling the backend
    }

    handleAuth('http://localhost:3000/auth/register', {
        username: user,
        password: pass
    });
};
    
document.getElementById('loginBtn').onclick = () => {
    const user = document.getElementById('username').value;
    const pass = document.getElementById('password').value;

    if (user.trim() === "" || pass.trim() === "") {
        showMessage("Please enter your credentials.");
        return; // Stop execution
    }

    handleAuth('http://localhost:3000/auth/login', {
        username: user,
        password: pass
    });
};
    