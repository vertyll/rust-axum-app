// Sends a file the way the back-end allows it: ask for an upload ticket, send the
// bytes to the signed link in it, then confirm, so the back-end checks the stored
// object before the file is listed. The texts come from the form's data attributes,
// already in the reader's language.
for (const form of document.querySelectorAll("form[data-upload]")) {
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    upload(form);
  });
}

async function upload(form) {
  const input = form.querySelector("input[type=file]");
  const submit = form.querySelector("[type=submit]");
  const progress = form.querySelector("[data-upload-progress]");
  const error = form.querySelector("[data-upload-error]");
  const file = input.files[0];
  if (!file) {
    return;
  }
  error.hidden = true;
  submit.disabled = true;
  progress.hidden = false;
  progress.value = 0;
  try {
    const ticket = await api(`${form.dataset.api}/upload-ticket`, {
      original_name: file.name,
      mime_type: file.type || "application/octet-stream",
      size: file.size,
    });
    await send(ticket.upload, file, progress);
    await api(`${form.dataset.api}/${ticket.file.id}/confirm`);
    window.location.assign(`${form.dataset.files}/${ticket.file.id}`);
  } catch (failure) {
    error.textContent = message(form, failure);
    error.hidden = false;
    progress.hidden = true;
    submit.disabled = false;
  }
}

async function api(url, body) {
  const response = await fetch(url, {
    method: "POST",
    headers: body ? { "Content-Type": "application/json" } : {},
    body: body ? JSON.stringify(body) : undefined,
  });
  const answer = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw answer;
  }
  return answer;
}

function send(link, file, progress) {
  return new Promise((resolve, reject) => {
    const request = new XMLHttpRequest();
    request.open(link.method, link.url);
    for (const [name, value] of Object.entries(link.headers)) {
      request.setRequestHeader(name, value);
    }
    request.upload.addEventListener("progress", (event) => {
      if (event.lengthComputable) {
        progress.value = event.loaded / event.total;
      }
    });
    request.addEventListener("load", () => (request.status < 300 ? resolve() : reject({})));
    request.addEventListener("error", () => reject({}));
    request.send(file);
  });
}

function message(form, problem) {
  const fieldCodes = Object.values(problem.errors ?? {}).flat();
  if (fieldCodes.includes("files.errors.too_large")) {
    const max = problem.args?.size?.max ?? "";
    return form.dataset.tooLarge.replace("{max}", max);
  }
  return form.dataset.failed;
}
