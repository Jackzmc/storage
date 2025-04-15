<script lang="ts">
	import '../app.css';
	import Nav from '$lib/components/nav.svelte';
	import Sidebar from '$lib/components/sidebar.svelte'
	
	let { children } = $props();
</script>

<Nav />
<div class="mx-5 columns">
	<div class="column pl-0 sidebar-column">
		<Sidebar />
	</div>
	<div class="column is-10">
		{@render children()}
	</div>
</div>

<style>
.columns, .sidebar-column {
	height: 100%;
}
.sidebar-column {
	border-right: 1px solid lightgray;
}
</style>
