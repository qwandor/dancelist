<table>
{% for (event, added) in diff.different %}
<tr><th colspan="7">{% if added %}Added{% else %}Removed{% endif %}</th></tr>
{% include "shared/event.md" %}
{% endfor %}
</table>

{{ diff.same }} events the same.
