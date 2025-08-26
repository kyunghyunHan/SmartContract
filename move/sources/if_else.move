module tester::IfElse {

    fun loops() {
        let mut i = 0;
        loop {
            i = i + 1;
            if (i == 3) { continue };
            if (i == 5) { break };
            let mut j = 0;
            while (j <10) {
                j = j + 1;
            }
           
        }
    }
}
