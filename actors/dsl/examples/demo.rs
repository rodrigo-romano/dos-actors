use gmt_dos_actors_dsl::actorscript;

fn main() {
    actorscript! {
        #[model]
        {
         (1: a<a_to_b> -> b),
         (10: a<a_to_c> -> &c<c_to_b> -> b)
        }
        // 10: a<to_c> -> c
    };
}
