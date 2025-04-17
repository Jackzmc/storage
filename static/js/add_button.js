document.addEventListener('DOMContentLoaded', () => {
    document.querySelector("#file-checkbox-all").addEventListener("input", (e) => {
        const checked = e.target.checked
        console.log("checked", checked)
        document.querySelectorAll(".file-checkbox").forEach(el => {
            el.checked = checked
        })
    })

    const readyhiddenItems = document.getElementsByClassName("readyhidden");
    for (let i = 0; i < readyhiddenItems.length; i++) {
        readyhiddenItems.item(i).classList.remove("readyhidden")
    }

    document.getElementById("modal-prompt-form")

  // Functions to open and close a modal
  function openModal($el) {
    $el.classList.add('is-active');
  }

  function closeModal($el) {
    $el.classList.remove('is-active');
  }

  function closeAllModals() {
    (document.querySelectorAll('.modal') || []).forEach(($modal) => {
      closeModal($modal);
    });
  }

  // Add a click event on various child elements to close the parent modal
  (document.querySelectorAll('.modal-background, .modal-close, .modal-card-head .delete, .modal-card-foot .button') || []).forEach(($close) => {
    const $target = $close.closest('.modal');

    $close.addEventListener('click', () => {
      closeModal($target);
    });
  });

  // Add a keyboard event to close all modals
  document.addEventListener('keydown', (event) => {
    if(event.key === "Escape") {
      closeAllModals();
    }
  });
});

function touch(type) {
    document.getElementById("modal-prompt").classList.add("is-active")
    document.getElementById("modal-prompt-input").focus()
    document.getElementById("modal-prompt-type").value = type
    if(type === "file") {
        document.getElementById("modal-prompt-input").setAttribute("placeholder", "myfile.txt")
        document.getElementById("modal-prompt-title").textContent = `Enter file name`
    } else {
        document.getElementById("modal-prompt-input").setAttribute("placeholder", "My Folder")
        document.getElementById("modal-prompt-title").textContent = `Enter folder name`
    }
}
async function touchSubmit(event) {
    event.preventDefault();
    const name = document.getElementById("modal-prompt-input").value
    const type = document.getElementById("modal-prompt-type").value
    console.debug("touch", type, name)

    const response = await fetch(`/api/library/${LIBRARY_ID}/touch?path=${LIBRARY_PATH}`, {
        headers: {
            "Content-Type": "application/json"
        },
        method: "POST",
        body: JSON.stringify({
            type,
            filename: name
        })
    })
    if(response.ok) {
        window.location.reload()
    } else {
        alert("error todo: better dialog")
    }
}
function upload(type) {

}
