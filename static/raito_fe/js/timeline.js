var timeline = [];

function timeline_error() {
  var error = document.createElement('div');

  error.className = 'status';

  error.innerHTML = `Error: Could not fetch statuses`;

  document.getElementById('inner-timeline').insertBefore(error, document.getElementById('inner-timeline').firstChild);
}

function prepare_status(status) {
  var new_status = document.createElement('div');

  new_status.className = 'status';

  new_status.innerHTML =
      `<div class="status-header">
              <img src="` + status.account.avatar + `" class="status-user-avatar">

              <div class="status-user-info">
                  <div class="status-user-info-displayname"><a href="` + status.account.url + `">` + status.account.display_name + `</a></div>
                  <div class="status-user-info-username"><a href="` + status.account.url + `">` + status.account.acct + `</a></div>
              </div>
          </div>

          <div class="status-content">` + status.content + `</div>

          <div class="status-info">
              <ul>
                  <li>
                      <div class="status-reply"><a class="status-reply-button">⤷</a><a class="status-info-count">` + status.replies_count + `</a>
                      </div>
                  </li>
                  <li>
                      <div class="status-favourite"><a class="status-favourite-button">♡</a><a class="status-info-count">` + status.favourites_count + `</a>
                      </div>
                  </li>
                  <li>
                      <div class="status-share"><a class="status-share-button">⟳</a><a class="status-info-count">` + status.reblogs_count + `</a>
                      </div>
                  </li>
              </ul>
              <div class="status-info-meta">
              <a href="` + status.url + `">View Thread</a>
              <a href="` + status.uri + `">` + status.created_at + `</a>
              </div>
          </div>`;

          document.getElementById('inner-timeline').insertBefore(new_status, document.getElementById('inner-timeline').firstChild);
}

function poll_timeline() {
  var api_endpoint = mastodon_api_base_uri + '/api/v1/timelines/public';
  fetch(api_endpoint)
  .then(response => {
    return response.json()
  })
  .then(data => {
    console.log(data);
    data.reverse();
    for (id in data)
    {
      if (!timeline.filter(function(e) { return e.id === data[id].id; }).length > 0)
      {
        timeline.push(data[id]);
        prepare_status(data[id]);
      }
    }
  })
  .catch(err => {
    console.log(err)
  })

    setTimeout(poll_timeline, 2000);
}
