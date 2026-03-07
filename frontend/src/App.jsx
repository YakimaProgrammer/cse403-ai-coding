import { useState } from 'react'

const API_URL = 'http://127.0.0.1:8000'

function App() {
  const [file, setFile] = useState(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState(null)
  const [result, setResult] = useState(null)
  const [showOptions, setShowOptions] = useState(false)
  const [options, setOptions] = useState({
    w1: 0, w2: 5, w3: 15, w4: 30, w5: 50,
    unlisted: 200, teammate: 50,
    size6: -1, size5: 25, size4: 50,
    minSize: 4, maxSize: 6
  })

  const handleOptionChange = (e) => {
    setOptions({ ...options, [e.target.name]: parseInt(e.target.value) || 0 })
  }

  const handleSubmit = async () => {
    if (!file) return
    setLoading(true)
    setError(null)
    setResult(null)

    const formData = new FormData()
    formData.append('file', file)
    formData.append('options', JSON.stringify(options))

    try {
      const resp = await fetch(`${API_URL}/solve`, { method: 'POST', body: formData })
      if (!resp.ok) {
        const err = await resp.json().catch(() => ({ detail: resp.statusText }))
        throw new Error(err.detail || `Error ${resp.status}`)
      }
      setResult(await resp.json())
    } catch (e) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="container">
      <h1>CSE403 Team Assignment Solver</h1>

      <div className="upload-section">
        <h2>Upload Student Preferences</h2>
        <p>Select a CSV file with student project preferences to generate team assignments.</p>
        <input type="file" accept=".csv" onChange={(e) => setFile(e.target.files[0])} />
        <br />
        <button className="btn" onClick={handleSubmit} disabled={!file || loading}>
          {loading ? 'Solving...' : 'Solve Team Assignments'}
        </button>

        <div style={{ marginTop: '1rem' }}>
          <button className="btn-secondary" onClick={() => setShowOptions(!showOptions)}>
            {showOptions ? 'Hide Options ▴' : 'More Options ▾'}
          </button>
        </div>

        {showOptions && (
          <div className="options-grid">
            <div className="option-group">
              <h4>Choice Weights</h4>
              <label>1st: <input type="number" name="w1" value={options.w1} onChange={handleOptionChange} /></label>
              <label>2nd: <input type="number" name="w2" value={options.w2} onChange={handleOptionChange} /></label>
              <label>3rd: <input type="number" name="w3" value={options.w3} onChange={handleOptionChange} /></label>
              <label>4th: <input type="number" name="w4" value={options.w4} onChange={handleOptionChange} /></label>
              <label>5th: <input type="number" name="w5" value={options.w5} onChange={handleOptionChange} /></label>
              <label>N/A: <input type="number" name="unlisted" value={options.unlisted} onChange={handleOptionChange} /></label>
            </div>
            <div className="option-group">
              <h4>Team Size Constraints</h4>
              <label>Min Size: <input type="number" name="minSize" value={options.minSize} onChange={handleOptionChange} /></label>
              <label>Max Size: <input type="number" name="maxSize" value={options.maxSize} onChange={handleOptionChange} /></label>
              <hr />
              <h4>Team Size Penalties</h4>
              <label>Size 6: <input type="number" name="size6" value={options.size6} onChange={handleOptionChange} /></label>
              <label>Size 5: <input type="number" name="size5" value={options.size5} onChange={handleOptionChange} /></label>
              <label>Size 4: <input type="number" name="size4" value={options.size4} onChange={handleOptionChange} /></label>
              <hr />
              <label>Teammate Split: <input type="number" name="teammate" value={options.teammate} onChange={handleOptionChange} /></label>
            </div>
          </div>
        )}
      </div>

      {loading && (
        <div className="loading">
          <div className="spinner" />
          <p>Solving team assignments... This may take up to 60 seconds.</p>
        </div>
      )}

      {error && (
        <div className="error">
          <strong>Error: </strong>{error}
        </div>
      )}

      {result && (
        <>
          <Metrics metrics={result.metrics} total={result.student_details.length} />
          <Teams teams={result.teams} />
          <StudentDetails students={result.student_details} />
        </>
      )}
    </div>
  )
}


function Metrics({ metrics, total }) {
  const dist = metrics.choice_distribution
  const counts = [dist['1'] || 0, dist['2'] || 0, dist['3'] || 0, dist['4'] || 0, dist['5'] || 0, dist['unlisted'] || 0]
  const labels = ['1st', '2nd', '3rd', '4th', '5th', 'N/A']
  const colors = ['choice-1', 'choice-2', 'choice-3', 'choice-4', 'choice-5', 'choice-unlisted']

  return (
    <div className="metrics-section">
      <h2>Assignment Quality Metrics</h2>

      <div className="metrics-grid">
        <div className="metric-card">
          <div className="value">{metrics.num_teams}</div>
          <div className="label">Teams Formed</div>
        </div>
        <div className="metric-card">
          <div className="value">{metrics.solver_status}</div>
          <div className="label">Solver Status</div>
        </div>
        <div className="metric-card">
          <div className="value">{metrics.teammate_satisfaction.percentage}%</div>
          <div className="label">Teammate Satisfaction</div>
        </div>
        <div className="metric-card">
          <div className="value">
            {metrics.teammate_satisfaction.satisfied}/{metrics.teammate_satisfaction.total_with_preferences}
          </div>
          <div className="label">With Preferred Teammate</div>
        </div>
      </div>

      <h3 style={{ margin: '1rem 0 0.5rem', color: '#555' }}>Choice Distribution</h3>
      <div className="choice-bar">
        {counts.map((count, i) => count > 0 && (
          <div
            key={i}
            className={`segment ${colors[i]}`}
            style={{ width: `${Math.max((count / total) * 100, 5)}%` }}
            title={`${labels[i]}: ${count}`}
          >
            {count}
          </div>
        ))}
      </div>
      <p style={{ fontSize: '0.85rem', color: '#777', marginTop: '0.3rem' }}>
        {labels.map((l, i) => `${l}: ${counts[i]}`).join(' | ')}
      </p>
    </div>
  )
}


function Teams({ teams }) {
  const sorted = Object.entries(teams).sort((a, b) => {
    const numA = parseInt(a[0].match(/\d+/)?.[0]) || 0
    const numB = parseInt(b[0].match(/\d+/)?.[0]) || 0
    return numA - numB || a[0].localeCompare(b[0])
  })

  return (
    <div className="teams-section">
      <h2>Team Assignments</h2>
      <table>
        <thead>
          <tr><th>Project</th><th>Size</th><th>Team Members</th></tr>
        </thead>
        <tbody>
          {sorted.map(([project, members]) => (
            <tr key={project}>
              <td><strong>{project}</strong></td>
              <td>{members.length}</td>
              <td>{members.join(', ')}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}


function StudentDetails({ students }) {
  const sorted = [...students].sort((a, b) => {
    const numA = parseInt(a.name.match(/\d+/)?.[0]) || 0
    const numB = parseInt(b.name.match(/\d+/)?.[0]) || 0
    return numA - numB
  })

  const rankBadge = (rank) => {
    const badgeClass = { 1: 'badge-1', 2: 'badge-2', 3: 'badge-3', 4: 'badge-4', 5: 'badge-5' }
    const suffix = ['st', 'nd', 'rd', 'th', 'th']
    return (
      <span className={`badge ${badgeClass[rank] || 'badge-unlisted'}`}>
        {rank <= 5 ? `${rank}${suffix[rank - 1]}` : 'N/A'}
      </span>
    )
  }

  return (
    <div className="student-details">
      <h2>Student Details</h2>
      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>NetID</th>
            <th>Assigned Project</th>
            <th>Choice Rank</th>
            <th>Preferred Teammate</th>
          </tr>
        </thead>
        <tbody>
          {sorted.map((s) => (
            <tr key={s.netid}>
              <td>{s.name}</td>
              <td>{s.netid}</td>
              <td>{s.assigned_project}</td>
              <td>{rankBadge(s.choice_rank)}</td>
              <td>
                <span className={`badge ${s.has_preferred_teammate ? 'badge-yes' : 'badge-no'}`}>
                  {s.has_preferred_teammate ? 'Yes' : 'No'}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

export default App
