const TABLE_LINK_RGX = /^\/tables\/([^\/]+)\/([^\/]+)/;

function main() {
  setCurrentMenuLink();
}

function setCurrentMenuLink() {
  let matches = window.location.pathname.match(TABLE_LINK_RGX);
  
  if (!matches) { return; }
  
  let [_, schema, table] = matches;
  let link = document.querySelector(`a[data-schema=${schema}][data-table=${table}`);
  
  if (!link) { return; }
  
  link.classList.add('current', 'disabled');
}

document.readyState === 'loading'
  ? document.addEventListener('DOMContentLoaded', main)
  : main();
