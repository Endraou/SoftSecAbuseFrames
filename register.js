const statusDiv = document.getElementById('status');
sessionStorage.removeItem('token');

// Configuration: Cooldown time in milliseconds (e.g., 30 seconds)
const COOLDOWN_TIME = 1500; 
let connexion_attempt = localStorage.getItem('connexionAttemp') | 0;

const showMessage = (msg, isError = true) => {
    statusDiv.innerText = msg;
    statusDiv.style.color = isError ? "#cf6679" : "#03dac6";
    statusDiv.style.marginTop = "10px";
};

// --- TIMER LOGIC ---

const getRemainingCooldown = () => {
    const lastAttempt = localStorage.getItem('lastAuthAttempt');
    if (!lastAttempt) return 0;

    const elapsed = Date.now() - parseInt(lastAttempt);
    return Math.max(0, COOLDOWN_TIME - elapsed);
};

const startCooldownTimer = () => {
    const timeLeft = getRemainingCooldown();
    if (timeLeft <= 0) return;

    // Disable buttons
    document.getElementById('regBtn').disabled = true;
    document.getElementById('loginBtn').disabled = true;

    const interval = setInterval(() => {
        const currentRemaining = getRemainingCooldown();
        
        if (currentRemaining <= 0) {
            clearInterval(interval);
            document.getElementById('regBtn').disabled = false;
            document.getElementById('loginBtn').disabled = false;
            showMessage("You can try again now.", false);
        } else {
            showMessage(`Too many attempts. Wait ${Math.ceil(currentRemaining / 1000)}s...`);
        }
    }, 1000);
};

// Check for existing cooldown on page load
window.onload = startCooldownTimer;

// --- AUTH LOGIC ---

async function handleAuth(url, data) {
    // Check if user is still on cooldown before even trying
    if (getRemainingCooldown() > 0) return;

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
            // Clear timer on successful login
            localStorage.removeItem('lastAuthAttempt'); 
            showMessage("Success! Redirecting...", false);
            setTimeout(() => { window.location.href = "main.html"; }, 1000);
        } else {
            // SET TIMER ON FAILURE
            localStorage.setItem('lastAuthAttempt', Date.now().toString());
            startCooldownTimer();
            showMessage(result.message || "Authentication failed.");
        }
    } catch (error) {
        showMessage("Connection failed. Is the server running?");
    }
};

document.getElementById('regBtn').onclick = () => {
    const user = document.getElementById('username').value;
    const pass = document.getElementById('password').value;

    // Check for empty fields
    if (user.trim() === "" || pass.trim() === "") {
        showMessage("Username and password are required.");
        return; 
    }

    // NEW: Check for minimum password length
    if (pass.length < 6) {
        showMessage("Password must be at least 6 characters long.");
        return; 
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
    