// TODO: Setting `.current` classes should happen on server?

function main() {
  setCurrentMenuLink();
  setCurrentMenuTab();

  let params = new URLSearchParams(window.location.search);

  document.querySelectorAll('c-table tbody tr').forEach(tr => {
    tr.addEventListener('click', evt => {
      if (evt.detail < 2) { return; }

      let schema = tr.getAttribute('data-schema');
      let table = tr.getAttribute('data-table');
      let recordId = tr.getAttribute('data-record');

      if (!schema || !table) { return; }

      window.location.href = `/tables/${schema}/${table}/records/${recordId}/edit`;
    })
  });

  document.querySelectorAll('c-table thead th').forEach(th => {
    let column = th.getAttribute('data-column');

    if (params.get('sort_column') == column) {
      th.classList.add('sorted');

      // Not guaranteed to have a direction in the search params, in which case
      // assume the default direction is `asc`
      th.classList.add(params.get('sort_direction') == 'desc' ? 'desc' : 'asc');

    }

    th.addEventListener('click', evt => {
      let previousColumn = params.get('sort_column');

      // Only toggle the sort direction if the column is being changed
      previousColumn == column
        ? toggleSortDirection(params)
        : params.set('sort_column', column);

      window.location.search = `?${params.toLocaleString()}`;
    });
  });
}

function toggleSortDirection(params) {
  // Sort direction defaults to `asc` if not in the params, so cannot rely
  // on that param being present
  params.get('sort_direction') == 'desc'
    ? params.set('sort_direction', 'asc')
    : params.set('sort_direction', 'desc');
}

function setCurrentMenuLink() {
  let rgx = /^\/tables\/([^\/]+)\/([^\/]+)/;
  let matches = window.location.pathname.match(rgx);
  
  if (!matches) { return; }
  
  let [_, schema, table] = matches;
  let link = document.querySelector(`c-sidebar a[data-schema=${schema}][data-table=${table}`);
  
  if (!link) { return; }
  
  link.classList.add('current', 'disabled');
}

function setCurrentMenuTab() {
  let link = document.querySelector(`c-content a[href="${window.location.pathname}"]`);

  if (!link) { return; }

  link.classList.add('current');
}

document.readyState === 'loading'
  ? document.addEventListener('DOMContentLoaded', main)
  : main();
