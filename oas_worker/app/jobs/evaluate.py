import jiwer

class Evaluation:
    def __init__(self, hypothesis, ground_truth):
        self.hypothesis = hypothesis
        self.ground_truth = ground_truth
        self.transformations = jiwer.Compose([
            jiwer.ToLowerCase(),
            jiwer.RemoveMultipleSpaces(),
            jiwer.RemoveWhiteSpace(replace_by_space=True),
            jiwer.RemovePunctuation()
        ])
    def evaluate(self, metrics):
        """ https://www.researchgate.net/publication/221478089_From_WER_and_RIL_to_MER_and_WIL_improved_evaluation_measures_for_connected_speech_recognition/link/00b4951f95799284d9000000/download
        """
        result = {}
        if "wer" in metrics:
            result["wer"] = jiwer.wer(
            self.ground_truth, 
            self.hypothesis, 
            truth_transform=self.transformations, 
            hypothesis_transform=self.transformations)
        if "mer" in metrics:
            result["mer"] = jiwer.mer(
            self.ground_truth, 
            self.hypothesis, 
            truth_transform=self.transformations, 
            hypothesis_transform=self.transformations)
        if "wil" in metrics:
            result["wil"] = jiwer.wil(
            self.ground_truth, 
            self.hypothesis, 
            truth_transform=self.transformations, 
            hypothesis_transform=self.transformations)

        return result
