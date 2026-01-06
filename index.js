// On cible les éléments du DOM
const noteInput = document.getElementById('noteInput');
const saveBtn = document.getElementById('saveBtn');

// Fonction pour simuler l'envoi vers un backend
const sendToBackend = async (content) => {
    console.log("Préparation de l'envoi au backend...");
    
    // Le 'payload' est l'objet contenant les données qu'on envoie
    const payload = {
        text: content,
        date: new Date().toISOString()
    };

    console.log("Données prêtes :", payload);
    
    // TODO: Ajouter le fetch() ici quand le backend sera prêt
    alert("Note enregistrée (en local) !");
};

// Écouteur d'événement sur le clic
saveBtn.addEventListener('click', () => {
    const content = noteInput.value;
    if (content.trim() !== "") {
        sendToBackend(content);
        noteInput.value = ""; // On vide le champ
    } else {
        alert("La note est vide !");
    }
});