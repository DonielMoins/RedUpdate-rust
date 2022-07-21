window.addEventListener("load", async () => {
    // (B) PARSE THE JSON STRING INTO OBJECT FIRST
    const response = await fetch('http://127.0.0.1:3030/', {
        method: "GET",
        headers: {
            'Content-Type': 'text/plain'
        }
    });
    data = await response.json();

    // console.table(data);

    // (C) GENERATE TABLE
    // (C1) CREATE EMPTY TABLE
    var table = document.createElement("table"),
        row, cellA, cellB;
    document.getElementById("demoC").appendChild(table);
    for (let key in data) {
        // (C2) ROWS & CELLS
        row = table.insertRow();
        cellA = row.insertCell();
        cellB = row.insertCell();

        // (C3) KEY & VALUE
        cellA.innerHTML = key;
        cellB.innerHTML = data[key];

    }
});
window.setInterval(async () => {
    // (B) PARSE THE JSON STRING INTO OBJECT FIRST
    const response = await fetch('http://127.0.0.1:3030/', {
        method: "GET",
        headers: {
            'Content-Type': 'text/plain'
        }
    });
    data = await response.json();

    document.getElementById("demoC").removeChild(document.getElementById("demoC").firstChild)
    var table = document.createElement("table"),
        row, cellA, cellB;
    document.getElementById("demoC").appendChild(table);
    for (let key in data) {
        // (C2) ROWS & CELLS
        row = table.insertRow();
        cellA = row.insertCell();
        cellB = row.insertCell();

        // (C3) KEY & VALUE
        cellA.innerHTML = key;
        cellB.innerHTML = data[key];

    }
}, 1000)

