<script lang="ts">
	import '../app.css';
	import Nav from '$lib/components/nav.svelte';
	import Sidebar from '$lib/components/sidebar.svelte'
	
	let { children } = $props();
</script>

<Nav />
<div class="mx-5">
	<div class="columns">
		<div class="column pl-0">
			<Sidebar />
		</div>
		<div class="column is-10">
			{@render children()}
		</div>
	</div>
</div>
