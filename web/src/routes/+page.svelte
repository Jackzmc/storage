<div class="">
    <h4 class="title is-4 is-inline">My Files</h4>
    <div class="is-pulled-right is-inline-block">
        <div class="buttons">
            <div class="button is-small">
                Display
            </div>
            <div class="button is-small">
                Sort
            </div>
        </div>
    </div>
    <hr class="my-2">
    <table class="table is-fullwidth">
        <thead>
            <tr>
                <td>Name </td>
                <td>Size </td>
                <td>Last Updated </td>
                <td>Owner </td>
            </tr>
        </thead>
    </table>
</div>
<FileList />

<script lang="ts">
import FileList from '$lib/components/filelist.svelte'
</script>