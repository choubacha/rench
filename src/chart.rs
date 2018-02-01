pub struct Chart {
    height: u32,
    fill: char,
    space: char,
}

impl Chart {
    pub fn new() -> Chart {
        Chart {
            height: 10,
            fill: '▌',
            space: ' ',
        }
    }

    pub fn height(mut self, h: u32) -> Chart {
        self.height = h;
        self
    }

    pub fn make<N>(&self, data: Vec<N>) -> String
    where
        N: Into<f64>,
    {
        let data: Vec<f64> = data.into_iter().map(|d| d.into()).collect();
        let (min, max): (f64, f64) = data.iter().fold((0., 0.), |(min, max), datum| {
            let datum = datum.clone();
            (
                if datum < min { datum } else { min },
                if max < datum { datum } else { max },
            )
        });
        let row_increment = (max - min) / self.height as f64;
        let mut ret = String::with_capacity(self.height as usize * data.len());
        for row in 0..self.height {
            let floor = max - ((row + 1) as f64 * row_increment);
            for datum in &data {
                ret.push(if *datum > floor {
                    self.fill.clone()
                } else {
                    self.space.clone()
                });
            }
            if row == 0 {
                ret.push_str(&format!(" {}", max));
            }
            if row == self.height - 1 {
                ret.push_str(&format!(" {}", min));
            }
            ret.push('\n');
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_makes_a_chart_of_default_height() {
        let chart = Chart::new().make(vec![1, 2, 3, 4, 3, 2, 1]);
        assert_eq!(
            chart,
            "   ▌    4
   ▌   
  ▌▌▌  
  ▌▌▌  
  ▌▌▌  
 ▌▌▌▌▌ 
 ▌▌▌▌▌ 
▌▌▌▌▌▌▌
▌▌▌▌▌▌▌
▌▌▌▌▌▌▌ 0
"
        );
    }

    #[test]
    fn it_can_change_the_height() {
        let chart = Chart::new().height(4).make(vec![1, 2, 3, 4, 3, 2, 1]);
        assert_eq!(
            chart,
            "   ▌    4
  ▌▌▌  
 ▌▌▌▌▌ 
▌▌▌▌▌▌▌ 0
"
        );
    }
}
