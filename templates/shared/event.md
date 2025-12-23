<tr class="event{% if event.multiday() %} multiday{% endif %}{% if event.cancelled %} cancelled{% endif %}">
<td class="event-dates">
{{ event.short_time() }}
</td>
<td class="event-name" title="{{ event.details.as_deref().unwrap_or_default() }}">
{% if let Some(link) = event.main_link() %}
<a href="{{ link }}">{{ event.name }}</a>
{% else %}
{{ event.name }}
{% endif %}
</td>
<td class="event-links">
{% for link in event.further_links() %}
<a href="{{ link.url }}">({{ link.short_name }})</a>
{% endfor %}
</td>
<td class="event-price">
{{ event.price.as_deref().unwrap_or_default() }}
</td>
<td class="event-location">
<a href="https://folkdance.page/?country={{ event.country|urlencode }}&city={{ event.city|urlencode }}">{{ event.city }}</a>,
{% if let Some(state) = event.state %}
<a href="https://folkdance.page/?country={{ event.country|urlencode }}&state={{ state|urlencode }}">{{ state }}</a>,
{% endif %}
<a href="https://folkdance.page/?country={{ event.country|urlencode }}">{{ event.country }}</a>
</td>
<td class="event-type">
{% if event.social %}
<a href="https://folkdance.page/?social=true" class="social" title="Social">S</a>
{% endif %}
{% if event.workshop %}
<a href="https://folkdance.page/?workshop=true" class="workshop" title="Workshop">W</a>
{% endif %}
</td>
<td class="event-styles">
{% for style in event.styles %}
<a class="dance-style {{ style.tag() }}" href="https://folkdance.page/?style={{ style.tag() }}">{{ style }}</a>
{% endfor %}
</td>
</tr>
{% if !event.bands.is_empty() || !event.callers.is_empty() %}
<tr class="details">
<td colspan="7">
{% for band in event.bands %}
<a href="https://folkdance.page/?band={{ band|urlencode }}" class="band">{{ band }}</a>
{% endfor %}
{% for caller_name in event.callers %}
<a href="https://folkdance.page/?caller={{ caller_name|urlencode }}" class="caller">{{ caller_name }}</a>
{% endfor %}
</td>
</tr>
{% endif %}
