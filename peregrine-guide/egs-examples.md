# EGS examples

These are concrete examples of the style language in action. Write your code sparsely and with long comments.

## Which program to run?

A bootstrap style language file configures which programs to run at which scale:

```
/* most zoomed out gene view: no structure, but with labels */
p := panel_new();
panel_add_tag(p,"local");
panel_add_track(p,"gene-pc-fwd");
panel_set_scale(p,19,50);
panel_set_max_scale_jump(p,6);
panel_apply(p,"self()","gene");
```

This is loaded at startup and allows you to specify which programs you want to run at what scale and howoften data needs recalculating (vs merely scaling).

## Retrieving

```
/* we always start by getting the data. self() is typical,
 * it meansretrieve it from the same URL as this source, but
 * you can specify other URLs.
 */
data := get_data("self()","gene",get_panel());
```

## Decoding

Decoding looks ugly on the page (especially if you don't use macros or sufficient whitespace) but it's essentially a series of nested function calls to turn input streams into your data streams.

```
/*

Get the starts from the data called "start" and apply delta_seq. This converts distances between starts to absolute values. For example (100,200,50) -> (100,300,350).

As starts come in order and their values are pretty evenly placed, this is better than sending absolute offsets especially later in a chromosome where there's a lot of constant MSB. For example

(... 10,234,500,123 10,234,600,456 10,235,000,678 ...)

takes much more space than

(20,124 18,421 12,349)

*/

start := delta_seq(data,"starts");

/*

It makes sense to send lengths rather than ends for the same reason: small and clustered. That can be undone by adding the start using the code below.

*/

end := start + delta_seq(data,"lengths");

/* strings come in LZ-compressed streams behind string_seq */

gene_id := string_seq(data,"gene_ids");

/*

"categories" (eg biotypes) come in a pair of data sources. The first is the list of keys used mapped to an abrbitrary number, the escond is a sequence of numbers. These are undone with classified_seq.

 */

gene_biotype := classified_seq(data,"gene_biotypes_keys","gene_biotypes_values");

```

## Styling

Here we illustrate how objects can be handled individually even without conditional branches.

Here genes are moved between tracks based on whether they are on the positive or negative strand and whether they are protein coding or not.

```
/* Generate as many zeroes as there are genes */
allotment_idx := len([gene_id]) (*) 0;

/* for every gene on the positive strand add two */
allotment_idx#[strand > 0] (+=) 2;

/* for every proteing-coding gene add one */
allotment_idx#[in(gene_biotype,["protein_coding"])] (+=) 1;

/* The values 0,1,2,3 can now be mapped to screen allotments */
allotment := index(allotment_idx,["gene-nonpc-rev","gene-pc-rev","gene-nonpc-fwd","gene-pc-fwd"]);
```

later on, the same index array is used to assign colours to the genes.

```
/* set some colours */
non_pc_colour := colour(250,250,250);
pc_colour := colour(128,128,128);

/* map genes to colours */
patina := patina_filled(index(allotment_idx,[non_pc_colour,pc_colour,non_pc_colour,pc_colour]));
```

Note how we can do different things with different members of an array even without branches. In reality, the colour constants and lists of biotypes wouldbe pulled out to compile-time config files (eg ini files) to allow easier customization.

## ZMenus

ZMenus are defined by templates and then data sources (streams of data generatedusing some method above) are applied to them.

```
/*

This isn't as scary as it looks. The zmenu function takes one argument, the pattern of the zmenu in the zmenu mini-syntax. In reality this would be pulled from a config.

*/

zmenu_tmpl := zmenu("[<light>Transcript</light> <strong>{transcript_id}</strong>] [<light>{transcript_biotype}</light>] [<light>{strand}</light>] / [<light>{transcript_id}</light>] [<light>{designation}</light>]");

/*

Once you have a template, you need to tell it what calues the keys of the various template parameters have. Presumably,
    designated_transript_id,
    designated_transcript_biotype, 
    strand_string, and 
    designated_transript_designation
are defined earlier in the file. 

*/

zmenus := patina_zmenu(

    // which template are we applying
    zmenu_tmpl,
    
    // which keys are we binding
    ["transcript_id","transcript_biotype","strand","designation"],
    
    // to what values
    [[designated_transcript_id],
     [designated_transcript_biotype],
     [strand_string],
     [designated_transcript_designation]][]);

```

## Drawing

Finally we can draw some stuff.

```
/* draw some rectangles */
rectangle_on_genome(start,end,5,patina,allotment,track);

/* "draw" some zmenus: two sets, transcript and gene */
rectangle_on_genome(start,end,8,tr_zmenu_patina,allotment,track);
rectangle_on_genome(start,end,8,ge_zmenu_patina,allotment,track);

/* draw the gene names underneath */
text_underneath(start,8,textpen,gene_name,allotment,track);
```