document.addEventListener('DOMContentLoaded', function() {
  const elements = document.querySelectorAll('.jsontree');
  elements.forEach(function(element) {
    const text = element.textContent;
    try {
      const jsonData = JSON.parse(text);
      element.innerHTML = JSONTree.create(jsonData);
      element.classList.remove('jsontree');
    } catch (e) {
      console.error('Failed to parse JSON:', e);
    }
  });
});
