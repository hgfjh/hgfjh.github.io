// Progressive enhancement: there is no iframe or remote thumbnail until a
// visitor activates the local button. The ordinary YouTube link always works.
for (const container of document.querySelectorAll(".paper-video")) {
  const id = container.dataset.videoId;
  const button = container.querySelector(".video-load");
  const player = container.querySelector(".video-player");

  // Only construct embed URLs from ordinary YouTube video IDs.
  if (!id || !/^[A-Za-z0-9_-]{11}$/.test(id) || !button || !player) {
    continue;
  }

  button.hidden = false;
  button.addEventListener("click", () => {
    const iframe = document.createElement("iframe");
    iframe.title = `Research introduction: ${container.dataset.videoTitle}`;
    iframe.src = `https://www.youtube-nocookie.com/embed/${id}?autoplay=1`;
    iframe.referrerPolicy = "strict-origin-when-cross-origin";
    iframe.allow = "autoplay; encrypted-media; picture-in-picture; fullscreen";
    iframe.allowFullscreen = true;

    player.replaceChildren(iframe);
    player.hidden = false;
    button.hidden = true;
    iframe.focus();
  }, { once: true });
}
