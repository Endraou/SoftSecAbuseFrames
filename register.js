const statusDiv = document.getElementById('status');

// Helper to show messages in the UI instead of alerts
const showMessage = (msg, isError = true) => {
    statusDiv.innerText = msg;
    statusDiv.style.color = isError ? "#cf6679" : "#03dac6"; // Red for error, Teal for success
    statusDiv.style.marginTop = "10px";
};
        
async function handleAuth(url, data) {
    showMessage("Processing...", false);

    try {
        const response = await fetch(url, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });

        const result = await response.json();

        if (response.ok) {
            localStorage.setItem('token', result.token);
            showMessage("Success! Redirecting...", false);
            setTimeout(() => {
                window.location.href = "index.html"; // Redirect to your notes page
            }, 1000);
        } else {
            // Backend errors (like "User already exists") are caught here
            showMessage(result.message || "Authentication failed.");
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
    